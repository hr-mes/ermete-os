# Proposta di Apprendimento: Simbiosi CodeGraph & Graphify

## 1. Classificazione
**Tipo:** Regola (Rule)
**Ambito:** Globale per tutti gli agenti (Analisi dell'architettura e mappatura del codice)

## 2. Razionale
L'utente ha stabilito che l'analisi del codice e la mappatura dell'architettura in Ermete OS (e in altri progetti complessi) non devono fare affidamento su un solo strumento parziale. **CodeGraph** eccelle nell'esplorazione esatta delle call-path (hop semantici tra funzioni), mentre **Graphify** fornisce una visione olistica basata su community detection (rilevamento dei God Node, raggruppamento architetturale cross-file). 
Usarli in "totale simbiosi e sinergia" garantisce che ogni futuro sub-agente (o agente principale) esegua audit completi, incrociando i dettagli microscopici del codice (CodeGraph) con l'impatto macroscopico delle dipendenze (Graphify).

## 3. Modifica Proposta
**Aggiunta di una nuova Global Rule** all'interno della configurazione globale dell'agente (es. `~/.gemini/rules/codegraph_graphify_symbiosis.md` o inserimento nel payload delle `<user_rules>` principali):

```xml
<RULE[codegraph_graphify_symbiosis]>
## Symbiosis of CodeGraph and Graphify
When tasked with codebase analysis, architectural mapping, or auditing, you MUST ALWAYS use **CodeGraph** and the **Graphify** skill in total symbiosis and synergy.
1. Use `Graphify` to generate the macro-level knowledge graph, identify "communities", and spot architectural bottlenecks (like God Nodes).
2. Use `CodeGraph` (`codegraph_explore` tool or shell CLI) to dive into the exact structural dependencies, call paths, and verbatim source lines.
Do not rely on one without the other for holistic architectural decisions.
</RULE[codegraph_graphify_symbiosis]>
```

## Azione Richiesta
Se confermi questa proposta, clicca su "Proceed" (Procedi). Provvederò a scrivere permanentemente questa regola nel tuo ambiente, affinché venga sempre caricata all'avvio in tutte le conversazioni future.
