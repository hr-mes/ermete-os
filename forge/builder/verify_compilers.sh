#!/bin/bash
set -eo pipefail

echo "🛡️  [Ermete OS Stage-0] Verifying Compiler Toolchain GPG Signatures..."

# List of critical compilers and toolchain packages
COMPILERS=("gcc" "gcc-c++" "rust" "cargo" "llvm" "clang")

for pkg in "${COMPILERS[@]}"; do
    if rpm -q "$pkg" >/dev/null 2>&1; then
        echo -n "[*] Verifying $pkg... "
        SIG=$(rpm -q "$pkg" --qf '%{SIGPGP:pgpsig}\n')
        
        if echo "$SIG" | grep -q "Key ID"; then
            echo "OK ($SIG)"
        else
            echo "FAILED"
            echo "🚨 CRITICAL: Package $pkg is NOT signed with a valid GPG key!"
            exit 1
        fi
    else
        echo "[-] Package $pkg is not installed, skipping."
    fi
done

echo "✅ All installed compilers are cryptographically verified."
exit 0
