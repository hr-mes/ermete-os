%global debug_package %{nil}
Name:           ermete-kernel
Version:        6.14.0
Release:        1.chimera2%{?dist}
Summary:        Ermete OS Chimera Kernel (Fedora Base + CachyOS BORE Scheduler + LLVM ThinLTO)

License:        GPL-2.0-only
URL:            https://github.com/hr-mes/ermete-forge

BuildRequires:  rpm-build rpmdevtools gcc gcc-c++ make cmake flex bison ncurses-devel elfutils-libelf-devel openssl-devel bc rsync tar wget curl cpio perl zstd git llvm clang lld ccache jq dwarves rust cargo

Provides:       kernel-bedrock = %{version}-%{release}
Provides:       kernel = %{version}-%{release}
Provides:       kernel-core = %{version}-%{release}
Provides:       kernel-modules = %{version}-%{release}
Provides:       kernel-modules-core = %{version}-%{release}
Provides:       kernel-modules-extra = %{version}-%{release}
Obsoletes:      kernel < %{version}-%{release}
Obsoletes:      kernel-core < %{version}-%{release}
Obsoletes:      kernel-modules < %{version}-%{release}
Obsoletes:      kernel-modules-core < %{version}-%{release}
Obsoletes:      kernel-modules-extra < %{version}-%{release}

%description
Ermete OS Chimera Kernel package metadata and build orchestration specification.
Integrates CachyOS BORE (Burst-Oriented Response Enhancer) scheduler, BBRv3,
LLVM ThinLTO optimization (-march=x86-64-v3), and zero-trust security hardening.

%prep
# Stub prep

%build
# Built via ThinLTO Clang toolchain and rpmbuild execution in build-local.sh

%install
# Kernel artifacts installed directly into rpmbuild workspace

%files

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 6.14.0-1.chimera2
- Initial release of ermete-kernel spec for Chimera kernel build pipeline
