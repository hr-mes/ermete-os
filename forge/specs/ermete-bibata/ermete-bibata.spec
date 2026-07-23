%global debug_package %{nil}
Name:           ermete-bibata-cursor
Version:        2.0.7
Release:        1%{?dist}
Summary:        Open source, compact, and material designed cursor set.
License:        GPLv3
URL:            https://github.com/ful1e5/Bibata_Cursor
Source0:        https://github.com/ful1e5/Bibata_Cursor/releases/download/v%{version}/Bibata-Modern-Classic.tar.xz

BuildArch:      noarch

%description
Bibata cursor theme (Modern Classic). Packaged for Ermete OS.

%prep
%setup -c -T
tar -xJf %{SOURCE0}

%build
# No build required for cursors

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/share/icons/Bibata-Modern-Classic
cp -r Bibata-Modern-Classic/* %{buildroot}/usr/share/icons/Bibata-Modern-Classic/

%files
/usr/share/icons/Bibata-Modern-Classic

%changelog
* Sun Jun 28 2026 Ermete Forge <forge@ermete.os> - 2.0.7-1
- Repackaged binary asset into RPM for zero-network OS build
