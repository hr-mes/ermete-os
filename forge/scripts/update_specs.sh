#!/bin/bash
set -euo pipefail

update_spec() {
    local repo=$1 spec_file=$2
    local latest_tag=$(curl -sL "https://api.github.com/repos/${repo}/releases/latest" | jq -r .tag_name | sed 's/^v//')
    if [ -z "$latest_tag" ] || [ "$latest_tag" == "null" ]; then
        echo "Could not fetch $repo"
        return
    fi
    local current_ver=$(grep -E '^Version:' "$spec_file" | awk '{print $2}')
    if [ "$latest_tag" != "$current_ver" ]; then
        echo "Updating $spec_file from $current_ver to $latest_tag"
        sed -i "s/^Version:.*$/Version:        $latest_tag/" "$spec_file"
    fi
}

update_spec "starship/starship" "specs/ermete-starship/ermete-starship.spec"
update_spec "InioX/matugen" "specs/ermete-matugen/ermete-matugen.spec"
update_spec "ful1e5/Bibata_Cursor" "specs/ermete-bibata/ermete-bibata.spec"
