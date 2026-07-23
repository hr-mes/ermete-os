%global debug_package %{nil}
Name:           ermete-nix-support
Version:        1.0.0
Release:        2%{?dist}
Summary:        Ermete OS ermete-nix-support
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

%description
Provides ermete-nix-support for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/lib/tmpfiles.d
mkdir -p %{buildroot}/usr/share/nix-initial-state/var/nix/profiles
ln -s var/nix %{buildroot}/nix
cp -a %{_sourcedir}/usr/lib/tmpfiles.d/* %{buildroot}/usr/lib/tmpfiles.d/ || true

%files
/nix
%dir /usr/share/nix-initial-state
%dir /usr/share/nix-initial-state/var
%dir /usr/share/nix-initial-state/var/nix
%dir /usr/share/nix-initial-state/var/nix/profiles
/usr/lib/tmpfiles.d/10-ermete-nix.conf

%changelog
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation with tmpfiles.d
