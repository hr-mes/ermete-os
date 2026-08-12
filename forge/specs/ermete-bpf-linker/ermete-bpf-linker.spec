Name:           ermete-bpf-linker
Version:        0.11.0
Release:        1%{?dist}
Summary:        Rust eBPF Linker for Ermete OS Live Patching

License:        MIT
URL:            https://github.com/aya-rs/bpf-linker
Source0:        https://github.com/aya-rs/bpf-linker/releases/download/v%{version}/bpf-linker-x86_64-unknown-linux-musl.tar.gz

%description
Pre-compiled bpf-linker to accelerate eBPF live-patching and security auditing without requiring cargo install at runtime.

%prep
%setup -q -c

%build
# Pre-compiled static binary (musl).

%install
mkdir -p %{buildroot}/usr/bin
install -m 0755 bpf-linker %{buildroot}/usr/bin/bpf-linker

%files
/usr/bin/bpf-linker

%changelog
* Wed Aug 12 2026 Ermete Architect <admin@ermete.os> - 0.11.0-1
- Real bpf-linker static binary packaging for air-gapped CI.
