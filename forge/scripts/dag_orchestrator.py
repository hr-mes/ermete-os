#!/usr/bin/env python3
"""
Ermete Forge DAG Orchestrator Engine
Calculates Directed Acyclic Graph (DAG) for RPM & Flatpak dependencies,
queries Redis distributed cache for node states, invalidates downstream dependencies,
and outputs topological matrix execution levels for parallel GitHub Actions execution.
"""

import sys
import os
import glob
import re
import json
import hashlib
from collections import defaultdict, deque

CONFIG_PATH = os.environ.get("CONFIG_PATH", "config/packages.json")
if not os.path.exists(CONFIG_PATH) and os.path.exists("forge/config/packages.json"):
    CONFIG_PATH = "forge/config/packages.json"

SPECS_DIR = os.environ.get("SPECS_DIR", "specs")
if not os.path.exists(SPECS_DIR) and os.path.exists("forge/specs"):
    SPECS_DIR = "forge/specs"



def compute_dir_hash(dir_path):
    """Calculates deterministic SHA256 for a directory."""
    hasher = hashlib.sha256()
    if not os.path.exists(dir_path):
        return hasher.hexdigest()[:16]
        
    for root, dirs, files in sorted(os.walk(dir_path)):
        for name in sorted(files):
            if name.startswith(".") or name.endswith(".swp"):
                continue
            filepath = os.path.join(root, name)
            try:
                with open(filepath, "rb") as f:
                    while chunk := f.read(65536):
                        hasher.update(chunk)
            except OSError:
                pass
    return hasher.hexdigest()[:16]

def parse_spec_dependencies(spec_path):
    """Extracts BuildRequires and Requires from a .spec file."""
    build_requires = set()
    requires = set()
    
    if not os.path.exists(spec_path):
        return build_requires, requires
        
    with open(spec_path, "r", encoding="utf-8", errors="ignore") as f:
        for line in f:
            line = line.strip()
            if line.startswith("BuildRequires:"):
                deps = line.split(":", 1)[1].strip()
                for dep in re.split(r'\s+|,', deps):
                    dep = re.sub(r'[><=].*', '', dep).strip()
                    if dep and not dep.startswith("%"):
                        build_requires.add(dep)
            elif line.startswith("Requires:"):
                deps = line.split(":", 1)[1].strip()
                for dep in re.split(r'\s+|,', deps):
                    dep = re.sub(r'[><=].*', '', dep).strip()
                    if dep and not dep.startswith("%"):
                        requires.add(dep)
                        
    return build_requires, requires

def load_package_manifest():
    """Loads packages.json single source of truth."""
    if os.path.exists(CONFIG_PATH):
        with open(CONFIG_PATH, "r") as f:
            return json.load(f)
    return {}

def build_dag(manifest):
    """Constructs the dependency graph and node metadata."""
    custom_pkgs = manifest.get("custom_packages", [])
    tier0 = manifest.get("custom_tier0", [])
    tier1 = manifest.get("custom_tier1", [])
    tier2 = manifest.get("custom_tier2", [])
    tier3 = manifest.get("custom_tier3", [])
    
    upstream_core = manifest.get("upstream_core", [])
    upstream_desktop = manifest.get("upstream_desktop", [])
    upstream_media = manifest.get("upstream_media", [])
    upstream_cli = manifest.get("upstream_cli", [])
    flatpaks = manifest.get("flatpaks", manifest.get("flatpak_packages", []))
    
    all_custom = set(custom_pkgs)
    all_upstream = set(upstream_core + upstream_desktop + upstream_media + upstream_cli)
    all_nodes = all_custom | all_upstream | set(flatpaks)
    
    graph = defaultdict(set)       # node -> set of nodes depending on node (outgoing edges)
    in_degree = defaultdict(int)   # node -> number of prerequisites
    prereqs = defaultdict(set)     # node -> set of nodes node depends on
    node_hashes = {}
    node_types = {}
    
    for node in all_nodes:
        in_degree[node] = 0
        if node in all_custom:
            node_types[node] = "custom"
        elif node in flatpaks:
            node_types[node] = "flatpak"
        else:
            node_types[node] = "upstream"
            
    # Add tier dependencies
    for t1 in tier1:
        for t0 in tier0:
            if t0 in all_nodes and t1 in all_nodes:
                graph[t0].add(t1)
                prereqs[t1].add(t0)
                
    for t2 in tier2:
        for t1 in tier1:
            if t1 in all_nodes and t2 in all_nodes:
                graph[t1].add(t2)
                prereqs[t2].add(t1)
                
    for t3 in tier3:
        for t2 in tier2:
            if t2 in all_nodes and t3 in all_nodes:
                graph[t2].add(t3)
                prereqs[t3].add(t2)

    # Parse spec files for direct dependencies
    for pkg in all_custom:
        spec_dir = os.path.join(SPECS_DIR, f"ermete-{pkg}")
        spec_files = glob.glob(os.path.join(spec_dir, "*.spec"))
        
        hash_val = compute_dir_hash(spec_dir)
        node_hashes[pkg] = hash_val
        
        if spec_files:
            build_reqs, reqs = parse_spec_dependencies(spec_files[0])
            for dep in build_reqs | reqs:
                clean_dep = dep.replace("ermete-", "")
                if clean_dep in all_nodes and clean_dep != pkg:
                    graph[clean_dep].add(pkg)
                    prereqs[pkg].add(clean_dep)

    for pkg in all_upstream:
        hash_val = hashlib.sha256(f"upstream-{pkg}".encode()).hexdigest()[:16]
        node_hashes[pkg] = hash_val
        
    for pkg in flatpaks:
        fp_dir = os.path.join("flatpaks", pkg)
        hash_val = compute_dir_hash(fp_dir)
        node_hashes[pkg] = hash_val

    for node in all_nodes:
        in_degree[node] = len(prereqs[node])
        
    return all_nodes, graph, prereqs, in_degree, node_hashes, node_types

def evaluate_dirty_nodes(all_nodes, graph, prereqs, node_hashes):
    """
    Reads local file cache for previous hashes.
    Marks node DIRTY if content hash changed OR if any upstream dependency is DIRTY.
    """
    dirty_nodes = set()
    transitive_hashes = {}
    
    in_deg = {n: len(prereqs[n]) for n in all_nodes}
    queue = deque([n for n in all_nodes if in_deg[n] == 0])
    topo_order = []
    
    while queue:
        curr = queue.popleft()
        topo_order.append(curr)
        for neighbor in graph[curr]:
            in_deg[neighbor] -= 1
            if in_deg[neighbor] == 0:
                queue.append(neighbor)

    os.makedirs(".cache", exist_ok=True)

    for node in topo_order:
        hasher = hashlib.sha256()
        hasher.update(node_hashes.get(node, "").encode())
        for parent in sorted(prereqs[node]):
            hasher.update(transitive_hashes.get(parent, "").encode())
        trans_hash = hasher.hexdigest()[:16]
        transitive_hashes[node] = trans_hash
        
        cached_val = None
        if os.path.exists(f".cache/{node}.hash"):
            try:
                with open(f".cache/{node}.hash", "r") as f:
                    cached_val = f.read().strip()
            except OSError:
                pass
                
        is_parent_dirty = any(parent in dirty_nodes for parent in prereqs[node])
        
        if cached_val != trans_hash or is_parent_dirty:
            dirty_nodes.add(node)
            # Write new hash to disk for future runs
            try:
                with open(f".cache/{node}.hash", "w") as f:
                    f.write(trans_hash)
            except OSError:
                pass

    return dirty_nodes, transitive_hashes

def partition_dag_levels(dirty_nodes, graph, prereqs, node_types):
    """
    Groups dirty nodes into topological execution levels (Level 0, Level 1, Level 2, Flatpaks).
    """
    level_0 = []
    level_1 = []
    level_2 = []
    flatpaks = []
    
    dirty_prereqs = {n: set(p for p in prereqs[n] if p in dirty_nodes) for n in dirty_nodes}
    dirty_in_degree = {n: len(dirty_prereqs[n]) for n in dirty_nodes}
    
    queue = deque([n for n in dirty_nodes if dirty_in_degree[n] == 0])
    level_map = {}
    
    for n in queue:
        level_map[n] = 0

    while queue:
        curr = queue.popleft()
        curr_lvl = level_map[curr]
        
        for neighbor in graph[curr]:
            if neighbor in dirty_nodes:
                level_map[neighbor] = max(level_map.get(neighbor, 0), curr_lvl + 1)
                dirty_in_degree[neighbor] -= 1
                if dirty_in_degree[neighbor] == 0:
                    queue.append(neighbor)

    for node in dirty_nodes:
        if node_types.get(node) == "flatpak":
            flatpaks.append(node)
        else:
            lvl = level_map.get(node, 0)
            if lvl == 0:
                level_0.append(node)
            elif lvl == 1:
                level_1.append(node)
            else:
                level_2.append(node)
                
    return level_0, level_1, level_2, flatpaks

def main():
    print("🧠 Forge DAG Architect initializing... (Local File Cache Enabled)")
    
    manifest = load_package_manifest()
    all_nodes, graph, prereqs, in_degree, node_hashes, node_types = build_dag(manifest)
    
    print(f"📊 DAG Topology built: {len(all_nodes)} nodes analyzed.")
    
    dirty_nodes, transitive_hashes = evaluate_dirty_nodes(
        all_nodes, graph, prereqs, node_hashes
    )
    
    level_0, level_1, level_2, flatpaks = partition_dag_levels(dirty_nodes, graph, prereqs, node_types)
    
    dag_plan = {
        "dirty_count": len(dirty_nodes),
        "level_0": level_0,
        "level_1": level_1,
        "level_2": level_2,
        "flatpaks": flatpaks
    }
    
    try:
        with open(".cache/dag_plan.json", "w") as f:
            json.dump(dag_plan, f)
    except OSError:
        pass
    
    has_changes = "true" if len(dirty_nodes) > 0 else "false"
    
    j_lvl0 = json.dumps(level_0)
    j_lvl1 = json.dumps(level_1)
    j_lvl2 = json.dumps(level_2)
    j_fp = json.dumps(flatpaks)
    
    print(f"🚀 DAG Execution Plan calculated:")
    print(f"  -> Level 0 Parallel Nodes ({len(level_0)}): {j_lvl0}")
    print(f"  -> Level 1 Parallel Nodes ({len(level_1)}): {j_lvl1}")
    print(f"  -> Level 2 Parallel Nodes ({len(level_2)}): {j_lvl2}")
    print(f"  -> Flatpak Parallel Nodes ({len(flatpaks)}): {j_fp}")
    print(f"  -> Has Changes: {has_changes}")

    github_output = os.environ.get("GITHUB_OUTPUT")
    if github_output:
        with open(github_output, "a") as f:
            f.write(f"dag_level_0={j_lvl0}\n")
            f.write(f"dag_level_1={j_lvl1}\n")
            f.write(f"dag_level_2={j_lvl2}\n")
            f.write(f"dag_flatpaks={j_fp}\n")
            f.write(f"dirty_count={len(dirty_nodes)}\n")
            f.write(f"has_changes={has_changes}\n")

if __name__ == "__main__":
    main()
