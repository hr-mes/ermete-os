%global debug_package %{nil}

Name:           ermete-cliphist
Version:        0.7.0
Release:        1%{?dist}
Summary:        Wayland clipboard manager
License:        GPL-3.0
URL:            https://github.com/sentriz/cliphist
Source0:        https://github.com/sentriz/cliphist/archive/v%{version}.tar.gz
BuildRequires:  golang
BuildRequires:  git
BuildRequires:  wayland-devel

Provides:       cliphist = %{version}-%{release}

%description
wayland clipboard manager. Packaged natively for Ermete OS.

%prep
%autosetup -n cliphist-%{version}

%build
# Inizializza go mod se non presente (in genere è presente)
export GOPATH=%{_builddir}/go
export GOCACHE=%{_builddir}/go-cache
go build -v -o cliphist

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 cliphist %{buildroot}%{_bindir}/cliphist

%files
%{_bindir}/cliphist

%changelog
* Fri Jul 10 2026 Ermete Forge <forge@ermete.os> - 0.7.0-1
- Initial encapsulation of cliphist for Ermete OS
