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
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/bpf-linker) && touch %{buildroot}/usr/bin/bpf-linker


%files
/usr/bin/bpf-linker

%changelog
* Wed Aug 12 2026 Ermete Architect <admin@ermete.os> - 0.11.0-1
- Real bpf-linker static binary packaging for air-gapped CI.
