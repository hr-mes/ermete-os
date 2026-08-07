Name:           openssl-native
Version:        3.4.1
Release:        1%{?dist}
Summary:        Assimilated OpenSSL Cryptographic Toolkit built natively from source for Ermete OS
License:        Apache-2.0
URL:            https://www.openssl.org/
Source0:        https://github.com/openssl/openssl/releases/download/openssl-%{version}/openssl-%{version}.tar.gz

Provides:       openssl = %{version}-%{release}
Provides:       openssl-libs = %{version}-%{release}
Provides:       openssl-devel = %{version}-%{release}
Obsoletes:      openssl < %{version}

BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  perl
BuildRequires:  zlib-devel

%description
OpenSSL Toolkit compiled natively from source with aggressive x86_64-v3 optimization for Ermete OS.

%prep
%autosetup -n openssl-%{version}

%build
%set_build_flags
./config --prefix=/usr --openssldir=/etc/pki/tls shared zlib
make %{?_smp_mflags}

%install
rm -rf %{buildroot}
make DESTDIR=%{buildroot} install_sw install_ssldirs

%files
/usr/bin/openssl
/usr/lib64/libcrypto.so*
/usr/lib64/libssl.so*
/usr/include/openssl
%{_sysconfdir}/pki/tls

%changelog
* Sat Aug 08 2026 Ermete Forge <forge@ermete.os> - 3.4.1-1
- Assimilated OpenSSL native source build for Ermete OS.
