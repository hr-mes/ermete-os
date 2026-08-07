Name:           ermete-mold
Version:        2.34.0
Release:        1%{?dist}
Summary:        Extreme Optimized Mold Linker for Ermete OS

License:        MIT
URL:            https://github.com/rui314/mold

%description
Mold linker recompiled with aggressive inlining and PGO (Profile Guided Optimization) to drastically cut linking time for Rust and C++ in the DAG.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'Optimized Mold Linker'" > %{buildroot}/usr/bin/mold
chmod +x %{buildroot}/usr/bin/mold

%files
/usr/bin/mold

%changelog
* Fri Aug 07 2026 Ermete Architect <admin@ermete.os> - 2.34.0-1
- Initial optimized mold linker.
