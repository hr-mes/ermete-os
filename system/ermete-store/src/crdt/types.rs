use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique node/actor identifier in the Ermete OS CRDT mesh.
pub type NodeId = String;

/// Timestamp for Last-Writer-Wins semantics with deterministic tie-breaking.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LwwTimestamp {
    /// Physical timestamp in milliseconds since UNIX epoch.
    pub timestamp: u64,
    /// Actor / Node identifier for deterministic tie-breaking when timestamps are equal.
    pub node_id: NodeId,
}

impl LwwTimestamp {
    /// Create a new timestamp with explicit values.
    pub fn new(timestamp: u64, node_id: impl Into<NodeId>) -> Self {
        Self {
            timestamp,
            node_id: node_id.into(),
        }
    }

    /// Create a timestamp representing current system time for a given node without unwrap.
    pub fn now(node_id: impl Into<NodeId>) -> Self {
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_millis() as u64,
            Err(_) => 0,
        };
        Self {
            timestamp,
            node_id: node_id.into(),
        }
    }
}

/// Last-Writer-Wins Register (LWW-Register) for scalar state values.
/// Merges by retaining the value with the latest clock timestamp (or node_id tie-breaker).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwwRegister<T> {
    pub value: T,
    pub clock: LwwTimestamp,
}

impl<T> LwwRegister<T> {
    /// Instantiate a new LWW-Register with value and initial clock.
    pub fn new(value: T, clock: LwwTimestamp) -> Self {
        Self { value, clock }
    }

    /// Instantiate a new LWW-Register stamped with the current time.
    pub fn now(value: T, node_id: impl Into<NodeId>) -> Self {
        Self {
            value,
            clock: LwwTimestamp::now(node_id),
        }
    }

    /// Update value if new clock is strictly higher than current clock.
    pub fn set(&mut self, value: T, clock: LwwTimestamp) -> bool {
        if clock > self.clock {
            self.value = value;
            self.clock = clock;
            true
        } else {
            false
        }
    }

    /// Immutable reference to register value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Extract inner value.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Clone> LwwRegister<T> {
    /// Convergent merge of two LWW-Registers.
    /// Retains the value whose clock is higher (ordering defined by timestamp then node_id).
    pub fn merge(&mut self, other: Self) {
        if other.clock > self.clock {
            self.value = other.value;
            self.clock = other.clock;
        }
    }

    /// Convergent merge against a reference to another LWW-Register.
    pub fn merge_ref(&mut self, other: &Self) {
        if other.clock > self.clock {
            self.value = other.value.clone();
            self.clock = other.clock.clone();
        }
    }
}

/// Unique tag attached to element addition events in OR-Sets.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tag {
    pub node_id: NodeId,
    pub seq: u64,
}

impl Tag {
    pub fn new(node_id: impl Into<NodeId>, seq: u64) -> Self {
        Self {
            node_id: node_id.into(),
            seq,
        }
    }
}

/// Add-Wins Observed-Removed Set (OR-Set) for dynamic collections.
/// Supports concurrent element additions and removals without conflicts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrSet<T: Ord + Clone> {
    /// Active element addition tags mapping.
    add_set: BTreeMap<T, BTreeSet<Tag>>,
    /// Observed removed tags (tombstones).
    tombstones: BTreeSet<Tag>,
    /// Local monotonic sequence counter for tag generation.
    seq_counter: u64,
}

impl<T: Ord + Clone> Default for OrSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> OrSet<T> {
    /// Initialize an empty OR-Set.
    pub fn new() -> Self {
        Self {
            add_set: BTreeMap::new(),
            tombstones: BTreeSet::new(),
            seq_counter: 0,
        }
    }

    /// Add an element to the OR-Set under a specific node authority.
    pub fn add(&mut self, element: T, node_id: &str) -> Tag {
        self.seq_counter = self.seq_counter.saturating_add(1);
        let tag = Tag::new(node_id, self.seq_counter);
        self.add_set
            .entry(element)
            .or_default()
            .insert(tag.clone());
        tag
    }

    /// Remove an element from the set by tombstoning all currently observed active tags for it.
    pub fn remove(&mut self, element: &T) -> bool {
        if let Some(tags) = self.add_set.get(element) {
            let active_tags: Vec<Tag> = tags.difference(&self.tombstones).cloned().collect();
            if !active_tags.is_empty() {
                for tag in active_tags {
                    self.tombstones.insert(tag);
                }
                self.compact();
                return true;
            }
        }
        false
    }

    /// Check if element is currently present in the set (has active non-tombstoned tag).
    pub fn contains(&self, element: &T) -> bool {
        if let Some(tags) = self.add_set.get(element) {
            tags.iter().any(|t| !self.tombstones.contains(t))
        } else {
            false
        }
    }

    /// Read all elements currently active in the set.
    pub fn read(&self) -> Vec<T> {
        self.add_set
            .iter()
            .filter_map(|(elem, tags)| {
                if tags.iter().any(|t| !self.tombstones.contains(t)) {
                    Some(elem.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Number of active elements in the set.
    pub fn len(&self) -> usize {
        self.read().len()
    }

    /// Check if set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convergent merge of two OR-Sets (Add-Wins join-semilattice operation).
    pub fn merge(&mut self, other: Self) {
        self.tombstones.extend(other.tombstones);

        for (elem, tags) in other.add_set {
            self.add_set.entry(elem).or_default().extend(tags);
        }

        if other.seq_counter > self.seq_counter {
            self.seq_counter = other.seq_counter;
        }

        self.compact();
    }

    /// Convergent merge against a reference to another OR-Set.
    pub fn merge_ref(&mut self, other: &Self) {
        self.tombstones.extend(other.tombstones.clone());

        for (elem, tags) in &other.add_set {
            self.add_set.entry(elem.clone()).or_default().extend(tags.clone());
        }

        if other.seq_counter > self.seq_counter {
            self.seq_counter = other.seq_counter;
        }

        self.compact();
    }

    /// Prune internal tombstones and inactive tags to maintain compact memory representation.
    pub fn compact(&mut self) {
        let tombstones = &self.tombstones;
        self.add_set.retain(|_, tags| {
            tags.retain(|tag| !tombstones.contains(tag));
            !tags.is_empty()
        });
    }
}

/// Serializable representation of Wi-Fi / Ethernet network configurations.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub ssid: String,
    pub security_type: String,
    pub is_autoconnect: bool,
}

impl NetworkConfig {
    pub fn new(ssid: impl Into<String>, security_type: impl Into<String>, is_autoconnect: bool) -> Self {
        Self {
            ssid: ssid.into(),
            security_type: security_type.into(),
            is_autoconnect,
        }
    }
}

/// Serializable representation of installed application package metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub app_id: String,
    pub version: String,
    pub channel: String,
}

impl PackageMetadata {
    pub fn new(app_id: impl Into<String>, version: impl Into<String>, channel: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            version: version.into(),
            channel: channel.into(),
        }
    }
}

/// Serializable state container representing the syncable OS configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrdtState {
    /// Local node identifier.
    pub node_id: NodeId,
    /// Desktop UI theme (LWW-Register).
    pub ui_theme: LwwRegister<String>,
    /// Desktop UI scaling factor percentage (LWW-Register).
    pub ui_scale_factor: LwwRegister<u32>,
    /// Known network configurations (OR-Set).
    pub known_networks: OrSet<NetworkConfig>,
    /// Installed application packages (OR-Set).
    pub installed_packages: OrSet<PackageMetadata>,
    /// Generic system configuration key-value store (Map of LWW-Registers).
    pub system_settings: BTreeMap<String, LwwRegister<String>>,
}

impl CrdtState {
    /// Create a default initial CRDT state for a node.
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        let nid = node_id.into();
        Self {
            ui_theme: LwwRegister::now("dark".to_string(), &nid),
            ui_scale_factor: LwwRegister::now(100, &nid),
            known_networks: OrSet::new(),
            installed_packages: OrSet::new(),
            system_settings: BTreeMap::new(),
            node_id: nid,
        }
    }

    /// Mathematical join-semilattice merge of two CrdtState structures.
    /// Guarantees state convergence across distributed devices without conflict resolution locks.
    pub fn merge(&mut self, other: Self) {
        self.ui_theme.merge(other.ui_theme);
        self.ui_scale_factor.merge(other.ui_scale_factor);
        self.known_networks.merge(other.known_networks);
        self.installed_packages.merge(other.installed_packages);

        for (key, other_reg) in other.system_settings {
            match self.system_settings.entry(key) {
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().merge(other_reg);
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(other_reg);
                }
            }
        }
    }

    /// Merge against a reference to another CrdtState.
    pub fn merge_ref(&mut self, other: &Self) {
        self.ui_theme.merge_ref(&other.ui_theme);
        self.ui_scale_factor.merge_ref(&other.ui_scale_factor);
        self.known_networks.merge_ref(&other.known_networks);
        self.installed_packages.merge_ref(&other.installed_packages);

        for (key, other_reg) in &other.system_settings {
            match self.system_settings.entry(key.clone()) {
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().merge_ref(other_reg);
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(other_reg.clone());
                }
            }
        }
    }
}
