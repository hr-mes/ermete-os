%global debug_package %{nil}
Name:           ermete-attestation
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Confidential Computing Attestation Daemon

License:        MIT
URL:            https://github.com/hr-mes/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config

%description
Confidential Computing Attestation Daemon for Ermete OS supporting AMD SEV-SNP and Intel TDX attestation reports over D-Bus.

%prep
%autosetup

%build
%set_build_flags
cargo generate-lockfile
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%files
%{_bindir}/%{name}

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-attestation spec
