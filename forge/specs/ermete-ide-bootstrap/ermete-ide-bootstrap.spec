%global debug_package %{nil}
Name:           ermete-ide-bootstrap
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS ermete-ide-bootstrap
License:        MIT
URL:            https://github.com/hr-mes/ermete-forge
BuildArch:      noarch

%description
Provides ermete-ide-bootstrap for Ermete OS.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/share/ermete-ide-bootstrap) && touch %{buildroot}/usr/share/ermete-ide-bootstrap


%files
%dir /usr/share/ermete-ide-bootstrap

%changelog
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
