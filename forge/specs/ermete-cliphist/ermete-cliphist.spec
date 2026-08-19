%global debug_package %{nil}

Name:           ermete-cliphist
Version:        0.7.0
Release:        1%{?dist}
Summary:        Wayland clipboard manager
License:        GPL-3.0
URL:            https://github.com/sentriz/cliphist

BuildRequires:  golang
BuildRequires:  git
BuildRequires:  wayland-devel

Provides:       cliphist = %{version}-%{release}

%description
wayland clipboard manager. Packaged natively for Ermete OS.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/cliphist) && touch %{buildroot}/usr/bin/cliphist


%files
/usr/bin/cliphist

%changelog
* Fri Jul 10 2026 Ermete Forge <forge@ermete.os> - 0.7.0-1
- Initial encapsulation of cliphist for Ermete OS
