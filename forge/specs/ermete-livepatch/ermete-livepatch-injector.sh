#!/bin/bash
set -e

# ermete-livepatch-injector
# Loads kpatch/livepatch modules into the kernel without reboot

PATCH_DIR="/usr/lib/modules/livepatch"

echo "Starting Ermete OS Live Patch Injection..."

if [ ! -d "$PATCH_DIR" ]; then
    echo "No live patches found in $PATCH_DIR"
    exit 0
fi

for patch_module in "$PATCH_DIR"/*.ko; do
    if [ -f "$patch_module" ]; then
        module_name=$(basename "$patch_module" .ko)
        echo "Injecting live patch: $module_name"
        
        # Check if already loaded
        if lsmod | grep -q "^$module_name "; then
            echo "Module $module_name is already loaded."
            continue
        fi
        
        # Load the module (via insmod which triggers ftrace routing in kpatch)
        echo "Loading $patch_module..."
        if insmod "$patch_module"; then
            echo "Successfully injected $module_name"
        else
            echo "Failed to inject $module_name" >&2
            exit 1
        fi
    fi
done

echo "Live patch injection complete."
