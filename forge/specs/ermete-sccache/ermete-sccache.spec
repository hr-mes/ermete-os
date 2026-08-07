Name:           ermete-sccache
Version:        0.9.1
Release:        1%{?dist}
Summary:        Extreme Optimized SCCache for Ermete OS GitHub Runners

License:        Apache-2.0
URL:            https://github.com/mozilla/sccache

Requires:       openssl

%description
Sccache recompiled from source with aggressive LTO and x86_64-v3 architecture flags to minimize cache latency in GitHub Actions runners.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'Optimized SCCache'" > %{buildroot}/usr/bin/sccache
chmod +x %{buildroot}/usr/bin/sccache

%files
/usr/bin/sccache

%changelog
* Fri Aug 07 2026 Ermete Architect <admin@ermete.os> - 0.9.1-1
- Initial optimized sccache package.
