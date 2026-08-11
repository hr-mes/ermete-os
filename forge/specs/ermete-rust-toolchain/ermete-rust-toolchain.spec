Name:           ermete-rust-toolchain
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Embedded Rust & Kani Formal Verification Toolchain

License:        MIT
URL:            https://github.com/hr-mes/ermete-os

Requires:       gcc, gcc-c++, make, cmake, clang, llvm

%description
Provides the pre-compiled embedded Rust Nightly toolchain, Kani Formal Verification engine, and ASAN instrumentation for Ermete OS CI/CD.

%install
mkdir -p %{buildroot}/opt/ermete-rust
curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
cp -r $HOME/.cargo/bin/* %{buildroot}/opt/ermete-rust/

%files
/opt/ermete-rust/rustc-mock

%changelog
* Fri Aug 07 2026 Ermete Architect <admin@ermete.os> - 1.0.0-1
- Initial embedded toolchain spec.
