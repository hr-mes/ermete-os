%global debug_package %{nil}
Name:           ermete-gatekeeper-rs
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Zero-Trust Gatekeeper (fanotify)

License:        GPLv3+
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.83.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros

%description
Ermete OS Zero-Trust binary execution gatekeeper using fanotify.

%prep
%setup -q

%build
%set_build_flags
%cargo_build

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
install -m 755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# systemd service
mkdir -p %{buildroot}%{_unitdir}
cat > %{buildroot}%{_unitdir}/%{name}.service <<EOF
[Unit]
Description=Ermete OS Zero-Trust Gatekeeper
After=network.target

[Service]
Type=simple
ExecStart=%{_bindir}/%{name}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

%files
%{_bindir}/%{name}
%{_unitdir}/%{name}.service

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release
