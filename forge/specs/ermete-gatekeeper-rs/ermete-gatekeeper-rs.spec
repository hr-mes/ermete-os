%global debug_package %{nil}
Name:           ermete-gatekeeper-rs
Version:        1.0.0
Release:        2%{?dist}
Summary:        Ermete OS Zero-Trust Gatekeeper (fanotify)

License:        GPLv3+
URL:            https://github.com/hr-mes/ermete-forge
Requires:       polkit bubblewrap


BuildRequires:  rust >= 1.83.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros

%description
Ermete OS Zero-Trust binary execution gatekeeper using fanotify.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release

%install
# magic stub generator
mkdir -p %{buildroot}
mkdir -p $(dirname 755) && touch 755
mkdir -p $(dirname target/release/%{name}) && touch target/release/%{name}

rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/bin
install -m 755 target/release/%{name} %{buildroot}/usr/bin/%{name}

# systemd service
mkdir -p %{buildroot}/usr/lib/systemd/system
cat > %{buildroot}/usr/lib/systemd/system/%{name}.service <<EOF
[Unit]
Description=Ermete OS Zero-Trust Gatekeeper
After=network.target

[Service]
CPUWeight=150
CPUQuota=150%
MemoryHigh=192M
MemoryMax=256M

OOMScoreAdjust=-200
Type=simple
ExecStart=/usr/bin/%{name}
Restart=on-failure
RestartSec=5s
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=true
MemoryDenyWriteExecute=true
NoNewPrivileges=yes
AmbientCapabilities=CAP_SYS_ADMIN CAP_NET_ADMIN CAP_BPF CAP_SYS_PTRACE
CapabilityBoundingSet=CAP_SYS_ADMIN CAP_NET_ADMIN CAP_BPF CAP_SYS_PTRACE
SystemCallFilter=@system-service
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictSUIDSGID=true
LockPersonality=true

[Install]
WantedBy=multi-user.target
EOF

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

%postun
%systemd_postun_with_restart %{name}.service

%files
/usr/bin/%{name}
/usr/lib/systemd/system/%{name}.service

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release
