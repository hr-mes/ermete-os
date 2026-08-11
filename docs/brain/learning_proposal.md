# Learning Proposal: CodeGraph & Graphify Symbiosis

## 1. Classification
**Type:** Rule  
**Scope:** Global across all agents (Codebase analysis & architectural mapping)  

## 2. Rationale
Codebase analysis and architectural mapping within Ermete OS must not rely on single isolated tools. **CodeGraph** excels at line-by-line call-path navigation and symbol extraction, while **Graphify** supplies macro-level topological analysis based on community detection (identifying God Nodes and cross-file dependencies).  
Utilizing both tools in full symbiosis ensures subagents execute holistic audits, cross-referencing verbatim code structures (CodeGraph) against systemic architectural impacts (Graphify).

## 3. Proposed Global Rule
Definition for inclusion in the global agent configuration (`~/.gemini/rules/codegraph_graphify_symbiosis.md`):

```xml
<RULE[codegraph_graphify_symbiosis]>
## Symbiosis of CodeGraph and Graphify
When tasked with codebase analysis, architectural mapping, or auditing, you MUST ALWAYS use **CodeGraph** and the **Graphify** skill in total symbiosis and synergy.
1. Use `Graphify` to generate the macro-level knowledge graph, identify "communities", and spot architectural bottlenecks (like God Nodes).
2. Use `CodeGraph` (`codegraph_explore` tool or shell CLI) to dive into the exact structural dependencies, call paths, and verbatim source lines.
Do not rely on one without the other for holistic architectural decisions.
</RULE[codegraph_graphify_symbiosis]>
```

## Action Requested
Upon confirmation, this rule will be permanently saved to the environment to load across all future agent sessions.
