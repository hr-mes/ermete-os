#!/bin/bash
# test_arbitrary_file_read.sh
# Tests the consequence of replacing realpath with podman cp to stdout.

# Setup mock environment
mkdir -p /tmp/mock_container_fs
echo '{"status": "ok"}' > /tmp/mock_container_fs/artifact.json

# The worker's code:
# podman cp "$(cat "$CIDFILE")":/workspace/artifact.json - | head -c 1048576 > "$TMP_FILE"
# We mock podman cp streaming a tarball to stdout
tar -cf - -C /tmp/mock_container_fs artifact.json | head -c 1048576 > test_artifact.tmp

# The worker's move command
mv -T test_artifact.tmp final_artifact.json

# Verification: The file is a TAR archive, NOT a JSON file!
if file final_artifact.json | grep -q "tar archive"; then
    echo "VULNERABILITY PROVEN: The extracted file is a TAR archive, breaking the JSON parser!"
    echo "The removal of the realpath/direct-read approach broke the ingestion pipeline."
fi
