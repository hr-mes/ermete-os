Name:           ermete-scudo
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Scudo Hardened Allocator Configuration

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge

BuildRequires:  systemd-rpm-macros
Requires:       compiler-rt

%description
Sets up Scudo standalone allocator via LD_PRELOAD globally for Ermete OS.

%prep
# No prep

%build
# No build

%install
mkdir -p %{buildroot}/etc
mkdir -p %{buildroot}%{_prefix}/lib/environment.d
mkdir -p %{buildroot}/usr/lib/systemd/system/greetd.service.d
mkdir -p %{buildroot}/usr/lib/systemd/system/ermete-llm.service.d

# Global LD_PRELOAD injection removed to preserve system stability and immutability

# Scudo Options
cat <<EOF > %{buildroot}%{_prefix}/lib/environment.d/10-scudo.conf
SCUDO_OPTIONS="ZeroContents=1:PatternFillRet=1:DeallocationTypeMismatch=1:DeleteSizeMismatch=1"
EOF

# Greetd override
cat <<EOF > %{buildroot}/usr/lib/systemd/system/greetd.service.d/override.conf
[Service]
LockPersonality=true
RestrictSUIDSGID=true
RestrictRealtime=true
MemoryDenyWriteExecute=true
ProtectControlGroups=true
ProtectKernelLogs=true
ProtectKernelModules=true
ProtectKernelTunables=true
Environment="LD_PRELOAD="
EOF

# Ermete LLM override
cat <<EOF > %{buildroot}/usr/lib/systemd/system/ermete-llm.service.d/override.conf
[Service]
LockPersonality=true
RestrictSUIDSGID=true
RestrictRealtime=true
MemoryDenyWriteExecute=true
ProtectControlGroups=true
ProtectKernelLogs=true
ProtectKernelModules=true
ProtectKernelTunables=true
Environment="LD_PRELOAD="
EOF

# Declarative symlink configuration via tmpfiles.d
mkdir -p %{buildroot}%{_prefix}/lib/tmpfiles.d
cat <<EOF > %{buildroot}%{_prefix}/lib/tmpfiles.d/10-scudo.conf
L+ /usr/lib64/libscudo.so - - - - /usr/lib64/clang/19/lib/linux/libclang_rt.scudo_standalone.so
EOF


%files
%{_prefix}/lib/environment.d/10-scudo.conf
%{_prefix}/lib/tmpfiles.d/10-scudo.conf
/usr/lib/systemd/system/greetd.service.d/override.conf
/usr/lib/systemd/system/ermete-llm.service.d/override.conf

%changelog
* Mon Aug 03 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release
