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
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/%{name}) && touch %{buildroot}/usr/bin/%{name}
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/%{name}.service) && touch %{buildroot}/usr/lib/systemd/system/%{name}.service


%files
/usr/bin/%{name}
/usr/lib/systemd/system/%{name}.service

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release
