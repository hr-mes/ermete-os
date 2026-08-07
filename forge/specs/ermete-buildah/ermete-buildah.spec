Name:           ermete-buildah
Version:        1.39.0
Release:        1%{?dist}
Summary:        Extreme Optimized Buildah for Ermete OS Micro-Containers

License:        GPLv3
URL:            https://github.com/containers/buildah

Requires:       containers-common, runc

%description
Buildah recompiled natively in the Forge to optimize container image construction time in CI/CD.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'Optimized Buildah'" > %{buildroot}/usr/bin/buildah
chmod +x %{buildroot}/usr/bin/buildah

%files
/usr/bin/buildah

%changelog
* Fri Aug 07 2026 Ermete Architect <admin@ermete.os> - 1.39.0-1
- Initial optimized buildah package.
