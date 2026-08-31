#!/bin/bash
set -euo pipefail

COMPONENT="${1:-all}"
TIME="${2:-60}"

for crate in specs/*/; do
    if [ -f "${crate}Cargo.toml" ] && [ -d "${crate}fuzz" ]; then
        comp=$(basename "$crate")
        if [ "$COMPONENT" = "all" ] || [ "$COMPONENT" = "$comp" ]; then
            echo "Fuzzing component $comp for ${TIME}s..."
            (cd "$crate" && cargo fuzz run fuzz_target_1 -- -max_total_time="$TIME" -sanitizer=address)
        fi
    fi
done
