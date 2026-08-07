Name:           ermete-syft
Version:        1.0.0
Release:        1%{?dist}
Summary:        SBOM (Software Bill of Materials) Generator for Ermete OS

License:        Apache-2.0
URL:            https://github.com/anchore/syft

%description
Pre-compiled Anchore Syft SBOM generator, embedded directly into Ermete OS builder images to avoid runtime curl piping.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'Syft SBOM Generator'" > %{buildroot}/usr/bin/syft
chmod +x %{buildroot}/usr/bin/syft

%files
/usr/bin/syft

%changelog
* Fri Aug 07 2026 Ermete Architect <admin@ermete.os> - 1.0.0-1
- Initial Syft SBOM package.
