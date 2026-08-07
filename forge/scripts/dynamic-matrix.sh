#!/bin/bash
set -euo pipefail

REGISTRY="ghcr.io"
OWNER="${GITHUB_REPOSITORY_OWNER:-hr-mes}"
REDIS_HOST="${REDIS_HOST:-redis}"
REDIS_PORT="${REDIS_PORT:-6379}"

echo "🌋 Executing Ermete Forge DAG Orchestration Engine..." >&2

# Run DAG computation Python engine with Redis distributed cache support
python3 scripts/dag_orchestrator.py --redis-host "$REDIS_HOST" --redis-port "$REDIS_PORT"
