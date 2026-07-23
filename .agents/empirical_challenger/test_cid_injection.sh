#!/bin/bash
TMP_WORKSPACE=$(mktemp -d)
CID_FILE="$TMP_WORKSPACE/cidfile"

# Attacker writes a malicious payload to CID_FILE
echo '$(touch /var/home/ermete/GEMINI/ermete/.agents/empirical_challenger/pwned)' > "$CID_FILE"

# Trap execution
podman rm -f "$(cat "$CID_FILE")" 2>/dev/null || true

if [ -f /var/home/ermete/GEMINI/ermete/.agents/empirical_challenger/pwned ]; then
    echo "VULNERABLE: Command injection via cidfile achieved!"
else
    echo "SAFE: Command injection prevented by quotes."
fi
rm -rf "$TMP_WORKSPACE"
