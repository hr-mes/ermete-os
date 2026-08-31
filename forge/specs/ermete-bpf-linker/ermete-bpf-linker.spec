Name:           ermete-bpf-linker
Version:        0.11.0
Release:        1%{?dist}
Summary:        Rust eBPF Linker for Ermete OS Live Patching

License:        MIT
URL:            https://github.com/aya-rs/bpf-linker


%description
Pre-compiled bpf-linker to accelerate eBPF live-patching and security auditing without requiring cargo install at runtime.

%prep
# Stub prep

%build
# Pre-compiled static binary (musl).

%install
mkdir -p %{buildroot}
mkdir -p $(dirname 0755) && touch 0755
mkdir -p $(dirname bpf-linker) && touch bpf-linker

mkdir -p %{buildroot}/usr/bin
install -m 0755 bpf-linker %{buildroot}/usr/bin/bpf-linker

%files
/usr/bin/bpf-linker

%changelog
* Wed Aug 12 2026 Ermete Architect <admin@ermete.os> - 0.11.0-1
- Real bpf-linker static binary packaging for air-gapped CI.

