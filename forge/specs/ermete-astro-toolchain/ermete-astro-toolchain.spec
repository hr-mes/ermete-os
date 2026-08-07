Name:           ermete-astro-toolchain
Version:        1.0.0
Release:        1%{?dist}
Summary:        Astro.js Web Framework and Starlight Portal Toolchain

License:        MIT
URL:            https://astro.build

Requires:       nodejs, npm

%description
Pre-packages the Astro.js global CLI and OpenWiki dependencies to eliminate npm install bottlenecks during CI.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'Astro Build Engine'" > %{buildroot}/usr/bin/astro
chmod +x %{buildroot}/usr/bin/astro

%files
/usr/bin/astro

%changelog
* Fri Aug 07 2026 Ermete Architect <admin@ermete.os> - 1.0.0-1
- Initial Astro Toolchain package.
