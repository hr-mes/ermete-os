podman run --rm -v $(pwd)/test_dnf.sh:/test_dnf.sh -it fedora:43 bash -c "
dnf install -y dnf5
mkdir -p /mnt/tier0-repo
# We don't have the repo mounted, but we can pull it with skopeo or buildah
buildah from --name test-tier0 ghcr.io/hr-mes/ermete-os-forge-tier0-repo:latest
mnt=\$(buildah mount test-tier0)
echo \"excludepkgs=kernel kernel-core kernel-modules kernel-modules-core kernel-modules-extra kernel-devel kernel-debug kernel-debug-devel kernel-debug-core kernel-debug-modules kernel-debug-modules-core kernel-debug-modules-extra\" >> /etc/dnf/dnf.conf
RAW_RPM_LIST=\$(find \$mnt -name '*.rpm' 2>/dev/null | grep -vE 'systemd-standalone-|.*-free-.*\.rpm|glibc32|kernel-debug|rpm-.*\.rpm|fedora-release-|selinux-policy|ermete-base-config' || true)
RPM_LIST=\"\"
for rpm in \$RAW_RPM_LIST; do
    if [[ \"\$rpm\" == *kmod-nvidia-* ]] && [[ \"\$rpm\" != *chimera* ]]; then
        continue;
    fi;
    RPM_LIST=\"\$RPM_LIST \$rpm\";
done
dnf5 install -y --allowerasing --setopt=install_weak_deps=False --setopt=tsflags=nodocs \$RPM_LIST
"
