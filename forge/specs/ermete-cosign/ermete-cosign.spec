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
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/cosign) && touch %{buildroot}/usr/bin/cosign


%files
/usr/bin/cosign

%changelog
* Wed Aug 12 2026 Ermete Architect <admin@ermete.os> - 2.4.0-1
- Initial Cosign binary packaging.
