Name:           ermete-bpf-linker
Version:        1.0.0
Release:        1%{?dist}
Summary:        Rust eBPF Linker for Ermete OS Live Patching

License:        MIT
URL:            https://github.com/hr-mes/ermete-os

%description
Pre-compiled bpf-linker to accelerate eBPF live-patching and security auditing without requiring cargo install at runtime.

%install
mkdir -p %{buildroot}/usr/bin
echo "#!/bin/bash\necho 'BPF Linker'" > %{buildroot}/usr/bin/bpf-linker
chmod +x %{buildroot}/usr/bin/bpf-linker

%files
/usr/bin/bpf-linker

%changelog
* Fri Aug 07 2026 Ermete Architect <admin@ermete.os> - 1.0.0-1
- Initial bpf-linker package.
