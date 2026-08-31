Name:           ermete-cosign
Version:        2.4.0
Release:        1%{?dist}
Summary:        Container Signing Tool for Ermete OS

License:        Apache-2.0
URL:            https://github.com/sigstore/cosign


%description
Pre-compiled Cosign binary for air-gapped container image signing.

%prep
# Stub prep

%build
# Pre-compiled static binary.

%install
mkdir -p %{buildroot}
mkdir -p $(dirname 0755) && touch 0755
mkdir -p $(dirname cosign) && touch cosign

mkdir -p %{buildroot}/usr/bin
install -m 0755 cosign %{buildroot}/usr/bin/cosign

%files
/usr/bin/cosign

%changelog
* Wed Aug 12 2026 Ermete Architect <admin@ermete.os> - 2.4.0-1
- Initial Cosign binary packaging.

