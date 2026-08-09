//! Mathematical CRDT (Conflict-free Replicated Data Type) module for Ermete OS.
//!
//! Provides deterministic state synchronization primitives across multi-device nodes:
//! - [`LwwRegister`]: Last-Writer-Wins Register for scalar values.
//! - [`OrSet`]: Add-Wins Observed-Removed Set for dynamic collections.
//! - [`CrdtState`]: Fully serializable Ermete OS distributed state.

pub mod types;

pub use types::{
    CrdtState, LwwRegister, LwwTimestamp, NetworkConfig, NodeId, OrSet, PackageMetadata, Tag,
};

/// Trait representing a CRDT join-semilattice primitive with convergent merge semantics.
pub trait Mergeable {
    /// Perform an in-place idempotent, commutative, and associative join merge.
    fn merge(&mut self, other: Self);
}

impl<T: Clone> Mergeable for LwwRegister<T> {
    fn merge(&mut self, other: Self) {
        LwwRegister::merge(self, other);
    }
}

impl<T: Ord + Clone> Mergeable for OrSet<T> {
    fn merge(&mut self, other: Self) {
        OrSet::merge(self, other);
    }
}

impl Mergeable for CrdtState {
    fn merge(&mut self, other: Self) {
        CrdtState::merge(self, other);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_register_convergence() {
        let mut r1 = LwwRegister::new("theme-a", LwwTimestamp::new(100, "node-1"));
        let r2 = LwwRegister::new("theme-b", LwwTimestamp::new(200, "node-2"));

        r1.merge(r2);
        assert_eq!(*r1.get(), "theme-b");

        // Deterministic tie-breaking on equal timestamps via node_id ordering
        let mut r3 = LwwRegister::new("val-node1", LwwTimestamp::new(300, "node-1"));
        let r4 = LwwRegister::new("val-node2", LwwTimestamp::new(300, "node-2"));

        r3.merge(r4.clone());
        assert_eq!(*r3.get(), "val-node2");

        // Commutativity test
        let mut r5 = LwwRegister::new("val-node2", LwwTimestamp::new(300, "node-2"));
        let r6 = LwwRegister::new("val-node1", LwwTimestamp::new(300, "node-1"));
        r5.merge(r6);
        assert_eq!(*r5.get(), "val-node2");
    }

    #[test]
    fn test_or_set_add_wins() {
        let mut set_a = OrSet::<String>::new();
        let mut set_b = OrSet::<String>::new();

        let elem = "wifi-home".to_string();
        set_a.add(elem.clone(), "node-1");

        // Merge set_a into set_b
        set_b.merge_ref(&set_a);
        assert!(set_b.contains(&elem));

        // Remove on set_a
        set_a.remove(&elem);
        assert!(!set_a.contains(&elem));

        // Concurrent add on set_b
        set_b.add(elem.clone(), "node-2");

        // Merge both
        set_a.merge(set_b);
        // Add-wins: node-2 addition tag survives node-1 removal
        assert!(set_a.contains(&elem));
    }

    #[test]
    fn test_crdt_state_serialization_and_merge() {
        let mut state_a = CrdtState::new("node-alpha");
        let mut state_b = CrdtState::new("node-beta");

        state_a.ui_theme = LwwRegister::new("light".to_string(), LwwTimestamp::new(500, "node-alpha"));
        state_b.ui_theme = LwwRegister::new("dark".to_string(), LwwTimestamp::new(600, "node-beta"));

        let net = NetworkConfig::new("ErmeteMesh", "WPA3", true);
        state_b.known_networks.add(net.clone(), "node-beta");

        state_a.merge(state_b);

        assert_eq!(*state_a.ui_theme.get(), "dark");
        assert!(state_a.known_networks.contains(&net));

        if let Ok(serialized) = serde_json::to_string(&state_a) {
            if let Ok(deserialized) = serde_json::from_str::<CrdtState>(&serialized) {
                assert_eq!(state_a, deserialized);
            } else {
                panic!("Deserialization failed");
            }
        } else {
            panic!("Serialization failed");
        }
    }
}
