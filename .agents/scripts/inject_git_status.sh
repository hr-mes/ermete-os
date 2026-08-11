#!/bin/bash
set -euo pipefail
cd /var/home/ermete/GEMINI/ermete-os || exit 0
STATUS=$(git status --short)

if [ -n "$STATUS" ]; then
  # Formatting JSON output securely
  # We escape newlines and quotes in the status string
  SAFE_STATUS=$(echo "$STATUS" | awk '{printf "%s\\n", $0}' | sed 's/"/\\"/g')
  cat <<EOF
{
  "injectSteps": [
    {
      "ephemeralMessage": "🚨 GIT AWARENESS HOOK:\nIl working tree ha modifiche non committate.\nStato attuale:\n$SAFE_STATUS\nNon dimenticare di gestirle o committarle."
    }
  ]
}
EOF
else
  cat <<EOF
{
  "injectSteps": []
}
EOF
fi
