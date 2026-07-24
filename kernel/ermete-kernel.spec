# ===================================================================
# Ermete OS Chimera Kernel Spec
# Base: CachyOS Linux Tree + Ermete OS Bedrock Enhancements
# ===================================================================

%define pkg_name ermete-kernel
%define kernel_version 6.14
%define pkg_release 1.chimera%{?dist}

# Enforce Clang / LLVM Toolchain & ThinLTO
%global toolchain clang
%define use_clang 1
%define use_thinlto 1

# Performance Flags: -O3 and -march=x86-64-v3
%global optflags -O3 -march=x86-64-v3 -pipe -fno-semantic-interposition
%global kcflags -O3 -march=x86-64-v3 -pipe

# Disable Debug Bloat & Unneeded Tools
%define _without_debug 1
%define _without_debuginfo 1
%define _without_doc 1
%define _without_perf 1
%define _without_tools 1
%define _without_selftests 1
%define debug_package %{nil}

# Binary payload compression
%define _binary_payload w1.zstdio
%define _source_payload w1.zstdio

Name:           %{pkg_name}
Version:        %{kernel_version}
Release:        %{pkg_release}
Summary:        The Ermete OS Chimera Kernel (CachyOS Base, BORE, NTSync, UKSM, LRNG, ThinLTO, -O3, x86-64-v3)
License:        GPLv2
URL:            https://github.com/CachyOS/linux-cachyos

Source0:        linux-cachyos.tar.gz
Source1:        ermete-bedrock.cfg

BuildRequires:  clang llvm lld
BuildRequires:  make cmake bison flex elfutils-libelf-devel openssl-devel openssl
BuildRequires:  ncurses-devel rsync tar bc cpio python3 zstd perl dwarves kmod

# Enforce Fedora OSTree and SELinux integration
Requires:       selinux-policy
Requires:       systemd
Requires:       dracut

%description
The Ermete OS Chimera Kernel combines CachyOS kernel performance improvements
(BORE scheduler, BBRv3, NTSync, LRNG, UKSM) with aggressive compiler optimization
(LLVM/Clang ThinLTO, -O3, -march=x86-64-v3), mitigations disabled by default for
maximum throughput, 1GB HugePages for KVM/AI workloads, and full Fedora OSTree/SELinux compatibility.

%prep
%setup -c -n linux-cachyos
# Prepare build tree and apply bedrock config overrides
if [ -f %{SOURCE1} ]; then
    cat %{SOURCE1} >> .config
fi

%build
export LLVM=1
export LLVM_IAS=1
export CC="${CC:-clang}"
export CXX="${CXX:-clang++}"
export LD=ld.lld
export KCFLAGS="%{kcflags}"
export KBUILD_CFLAGS="%{kcflags}"

make olddefconfig
make -j$(nproc) bzImage modules

%package devel
Summary:        Development package for building kernel modules to match the Chimera kernel
Provides:       kernel-devel = %{version}-%{release}

%description devel
This package provides kernel headers and makefiles sufficient to build modules
against the %{pkg_name} package.

%install
mkdir -p %{buildroot}/boot

make INSTALL_MOD_PATH=%{buildroot} modules_install
KREL=$(make -s kernelrelease)

cp arch/x86/boot/bzImage %{buildroot}/boot/vmlinuz-$KREL-chimera
cp System.map %{buildroot}/boot/System.map-$KREL-chimera
cp .config %{buildroot}/boot/config-$KREL-chimera

# Install kernel-devel source tree for module building
mkdir -p %{buildroot}/usr/src/kernels/$KREL-chimera
echo "Copying kernel headers and makefiles..."
rsync -a --prune-empty-dirs --include '*/' --include '*.h' --include 'Makefile*' --include 'Kbuild*' --include 'Kconfig*' --include 'Module.symvers' --include '.config' --include 'scripts/***' --include 'tools/***' --exclude '*' ./ %{buildroot}/usr/src/kernels/$KREL-chimera/
# Copy some missing binaries required by akmods
cp -a scripts %{buildroot}/usr/src/kernels/$KREL-chimera/

# Fix the symlink in /lib/modules so it points to our new kernel-devel directory
rm -f %{buildroot}/lib/modules/$KREL/build
rm -f %{buildroot}/lib/modules/$KREL/source
ln -snf /usr/src/kernels/$KREL-chimera %{buildroot}/lib/modules/$KREL/build
ln -snf /usr/src/kernels/$KREL-chimera %{buildroot}/lib/modules/$KREL/source

%files
/boot/vmlinuz-*
/boot/System.map-*
/boot/config-*
/lib/modules/*

%files devel
/usr/src/kernels/*

%changelog
* Thu Jul 23 2026 Kernel Alchemist <kernel@ermeteos.org> - 6.14-1.chimera
- Added kernel-devel package for out-of-tree module compilation (Nvidia)
- Fix broken symlinks in /lib/modules/*/build
