Name:           ermete-syft
Version:        1.10.0
Release:        1%{?dist}
Summary:        SBOM (Software Bill of Materials) Generator for Ermete OS

License:        Apache-2.0
URL:            https://github.com/anchore/syft


%description
Pre-compiled Anchore Syft SBOM generator, packaged directly for the Ermete OS builder images to avoid runtime network calls.

%prep
# Stub prep

%build
# Pre-compiled binary, nothing to build.

%install
mkdir -p %{buildroot}/usr/bin
install -m 0755 syft %{buildroot}/usr/bin/syft

%files
/usr/bin/syft

%changelog
* Wed Aug 12 2026 Ermete Architect <admin@ermete.os> - 1.10.0-1
- Real Syft binary packaging to eradicate curl-piping.
