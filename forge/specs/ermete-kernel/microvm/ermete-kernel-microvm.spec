# Kernel guest per le MicroVM di Ermete OS (docs/architecture/doc_kernel_build.md,
# sezione 9). Stessa sorgente e stesso pin del kernel principale, secondo config: non
# passa dal packaging Fedora e produce vmlinux (quello che Firecracker e cloud-hypervisor
# caricano), bzImage e il config in /usr/lib/ermete/microvm/. build.sh passa l'albero
# preparato da `rpmbuild -bp` del kernel.spec e la directory oggetto gia' configurata
# (x86_64_defconfig + kvm_guest.config + microvm/kernel-local): qui c'e' solo la
# compilazione con O=, cosi' l'albero resta pulito per il kernel principale.
# Niente strip ne' debuginfo automatici: vmlinux si spoglia del DWARF a mano, tenendo
# simboli e .BTF.
%global debug_package %{nil}
%global __os_install_post %{nil}
%global microvm_dir /usr/lib/ermete/microvm

Name:           ermete-kernel-microvm
Version:        %{kversion}
Release:        %{krelease}
Summary:        Kernel guest di Ermete OS per le MicroVM
License:        GPL-2.0-only WITH Linux-syscall-note
URL:            https://github.com/hr-mes/ermete-os
ExclusiveArch:  x86_64
# La toolchain del kernel principale (builder/Containerfile): clang e lld, pahole per il BTF.
BuildRequires:  clang lld llvm make dwarves

%description
Il kernel che hypervisor-daemon avvia nelle MicroVM (Firecracker, cloud-hypervisor):
x86_64_defconfig + kvm_guest.config + microvm/kernel-local (virtio, 9p e virtiofs,
EROFS, dm-verity, BPF con BTF; nessun driver fisico, nessun modulo), compilato con
clang e kCFI dalla stessa sorgente del kernel principale.

%build
make -C %{kernel_tree} O=%{objdir} %{make_opts} -j%{_smp_build_ncpus} vmlinux bzImage

%install
install -d %{buildroot}%{microvm_dir}
llvm-strip --strip-debug -o %{buildroot}%{microvm_dir}/vmlinux %{objdir}/vmlinux
install -m 644 %{objdir}/arch/x86/boot/bzImage %{buildroot}%{microvm_dir}/bzImage
install -m 644 %{objdir}/.config %{buildroot}%{microvm_dir}/config
make -s -C %{kernel_tree} O=%{objdir} %{make_opts} kernelrelease > %{buildroot}%{microvm_dir}/release

%files
%{microvm_dir}/

%changelog
