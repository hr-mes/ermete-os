#!/bin/bash
set -euo pipefail

OWNER="${1:-hr-mes}"

PACKAGES=$(gh api --paginate "/users/${OWNER}/packages?package_type=container" | jq -r '.[].name' 2>/dev/null || echo "")
if [ -z "$PACKAGES" ]; then
    echo "No container packages found."
    exit 0
fi

for PACKAGE in $PACKAGES; do
    ENC_PACKAGE=$(jq -rn --arg x "$PACKAGE" '$x|@uri')
    VERSIONS_JSON=$(gh api --paginate "/users/${OWNER}/packages/container/${ENC_PACKAGE}/versions" 2>/dev/null || echo "[]")
    KEEP_IDS=$(echo "$VERSIONS_JSON" | jq -r '[.[] | select(.metadata.container.tags | length > 0)] | sort_by(.created_at) | reverse | .[0:2] | .[].id')
    echo "$VERSIONS_JSON" | jq -c '.[]' | while read -r version; do
        VERSION_ID=$(echo "$version" | jq -r '.id')
        TAG_COUNT=$(echo "$version" | jq -r '.metadata.container.tags | length')
        if [[ "$TAG_COUNT" -eq 0 ]]; then
            echo "Deleting untagged version $VERSION_ID..."
            gh api -X DELETE "/users/${OWNER}/packages/container/${ENC_PACKAGE}/versions/${VERSION_ID}" || true
        elif ! echo "$KEEP_IDS" | grep -q "^$VERSION_ID$"; then
            echo "Deleting old version $VERSION_ID..."
            gh api -X DELETE "/users/${OWNER}/packages/container/${ENC_PACKAGE}/versions/${VERSION_ID}" || true
        fi
    done
done
