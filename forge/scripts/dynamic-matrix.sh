#!/bin/bash
set -euo pipefail

REGISTRY="ghcr.io"
OWNER="${GITHUB_REPOSITORY_OWNER:-hr-mes}"

echo "🌋 Executing Ermete Forge DAG Orchestration Engine..." >&2

# Run DAG computation Python engine with local caching
python3 scripts/dag_orchestrator.py
