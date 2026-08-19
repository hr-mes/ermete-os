Name:           ermete-livepatch
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Kernel Live Patch

License:        GPLv2
URL:            https://github.com/ermete-os/ermete-livepatch


Requires:       kpatch
Requires:       kmod

%description
Live patches for Ermete OS kernel (Zero-Downtime ring-0 patching).

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/lib/modules/livepatch/) && touch %{buildroot}/usr/lib/modules/livepatch/


%files
/usr/lib/modules/livepatch/

%changelog
* Mon Aug 03 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial live patch package structure
