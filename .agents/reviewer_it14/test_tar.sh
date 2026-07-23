#!/bin/bash
cat /dev/null | tar -xO > test.out
echo "exit code: $?"
cat test.out
