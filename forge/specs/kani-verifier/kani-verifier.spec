Name:           kani-verifier
Version:        0.67.0
Release:        1%{?dist}
Summary:        Kani Rust Formal Verification Engine

License:        Apache-2.0 OR MIT
URL:            https://github.com/model-checking/kani
Source0:        https://github.com/model-checking/kani/releases/download/kani-%{version}/kani-%{version}-x86_64-unknown-linux-gnu.tar.gz

Requires:       rustc, cargo

%description
Kani Rust Verifier is a bit-precise model checker for Rust code.
Packaged as an offline bundle to prevent 503 errors from GitHub during CI/CD.

%prep
%setup -q -n kani-%{version}

%build
# Pre-compiled bundle containing kani-compiler, cargo-kani, etc.

%install
mkdir -p %{buildroot}/opt/kani
cp -r * %{buildroot}/opt/kani/

mkdir -p %{buildroot}/usr/bin
ln -s /opt/kani/bin/cargo-kani %{buildroot}/usr/bin/cargo-kani
ln -s /opt/kani/bin/kani %{buildroot}/usr/bin/kani

%files
/opt/kani/
/usr/bin/cargo-kani
/usr/bin/kani

%changelog
* Wed Aug 12 2026 Kani Forge Architect <admin@ermete.os> - 0.67.0-1
- Real Kani offline bundle to eradicate GitHub 503 failures.
