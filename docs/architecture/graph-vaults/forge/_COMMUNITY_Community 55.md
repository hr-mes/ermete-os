---
type: community
cohesion: 0.21
members: 14
---

# Community 55

**Cohesion:** 0.21 - loosely connected
**Members:** 14 nodes

## Members
- [[Calculates deterministic SHA256 for a directory.]] - rationale - scripts/dag_orchestrator.py
- [[Constructs the dependency graph and node metadata.]] - rationale - scripts/dag_orchestrator.py
- [[Extracts BuildRequires and Requires from a .spec file.]] - rationale - scripts/dag_orchestrator.py
- [[Groups dirty nodes into topological execution levels (Level 0, Level 1, Level 2,]] - rationale - scripts/dag_orchestrator.py
- [[Loads packages.json single source of truth.]] - rationale - scripts/dag_orchestrator.py
- [[Reads local file cache for previous hashes.     Marks node DIRTY if content hash]] - rationale - scripts/dag_orchestrator.py
- [[build_dag()]] - code - scripts/dag_orchestrator.py
- [[compute_dir_hash()]] - code - scripts/dag_orchestrator.py
- [[dag_orchestrator.py]] - code - scripts/dag_orchestrator.py
- [[evaluate_dirty_nodes()]] - code - scripts/dag_orchestrator.py
- [[load_package_manifest()]] - code - scripts/dag_orchestrator.py
- [[main()]] - code - scripts/dag_orchestrator.py
- [[parse_spec_dependencies()]] - code - scripts/dag_orchestrator.py
- [[partition_dag_levels()]] - code - scripts/dag_orchestrator.py

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Community_55
SORT file.name ASC
```
