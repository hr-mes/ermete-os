Name:           ermete-livepatch
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Kernel Live Patch

License:        GPLv2
URL:            https://github.com/ermete-os/ermete-livepatch
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  kpatch-build
Requires:       kpatch
Requires:       kmod

%description
Live patches for Ermete OS kernel (Zero-Downtime ring-0 patching).

%prep
%setup -q

%build
# kpatch-build ...

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/lib/modules/livepatch/
# cp build/livepatch/*.ko $RPM_BUILD_ROOT/usr/lib/modules/livepatch/

%files
/usr/lib/modules/livepatch/

%changelog
* Mon Aug 03 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial live patch package structure
