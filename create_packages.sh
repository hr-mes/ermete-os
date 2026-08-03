#!/bin/bash
set -e
mkdir -p forge/specs/ermete-tetragon/SOURCES
mkdir -p forge/specs/ermete-scudo/SOURCES
mkdir -p forge/specs/ermete-qa/SOURCES

# Move tetragon configs
mv system/tetragon.service forge/specs/ermete-tetragon/SOURCES/
mv system/tetragon.yaml forge/specs/ermete-tetragon/SOURCES/
mkdir -p forge/specs/ermete-tetragon/SOURCES/tetragon.tp.d
mv system/tetragon.tp.d/sys_execve.yaml forge/specs/ermete-tetragon/SOURCES/tetragon.tp.d/
rmdir system/tetragon.tp.d || true

# Move TPM configs to ermete-secure-boot
mkdir -p forge/specs/ermete-secure-boot/SOURCES/usr/libexec/ermete
mkdir -p forge/specs/ermete-secure-boot/SOURCES/usr/lib/systemd/system/systemd-pcrphase-sysinit.service.d
mv system/scripts/ermete-tpm-rollback-check.sh forge/specs/ermete-secure-boot/SOURCES/usr/libexec/ermete/
mv system/scripts/ermete-tpm-rollback-update.sh forge/specs/ermete-secure-boot/SOURCES/usr/libexec/ermete/
mv system/services/ermete-tpm-rollback-check.service forge/specs/ermete-secure-boot/SOURCES/usr/lib/systemd/system/
mv system/services/ermete-tpm-rollback-update.service forge/specs/ermete-secure-boot/SOURCES/usr/lib/systemd/system/
mv system/services/10-rollback-check.conf forge/specs/ermete-secure-boot/SOURCES/usr/lib/systemd/system/systemd-pcrphase-sysinit.service.d/

# Move QA script
mv system/test-nvidia-modules.sh forge/specs/ermete-qa/SOURCES/

# We'll create spec files next
