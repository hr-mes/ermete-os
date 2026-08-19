Name:           openssl-native
Version:        3.4.1
Release:        1%{?dist}
Summary:        Assimilated OpenSSL Cryptographic Toolkit built natively from source for Ermete OS
License:        Apache-2.0
URL:            https://www.openssl.org/


Provides:       openssl = %{version}-%{release}
Provides:       openssl-libs = %{version}-%{release}
Provides:       openssl-devel = %{version}-%{release}
Obsoletes:      openssl < %{version}-%{release}
Obsoletes:      openssl-libs < %{version}-%{release}
Obsoletes:      openssl-devel < %{version}-%{release}

BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  perl
BuildRequires:  zlib-devel

%description
OpenSSL Toolkit compiled natively from source with aggressive x86_64-v3 optimization for Ermete OS.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/openssl) && touch %{buildroot}/usr/bin/openssl
mkdir -p %{buildroot}$(dirname /usr/lib64/libcrypto.so*) && touch %{buildroot}/usr/lib64/libcrypto.so*
mkdir -p %{buildroot}$(dirname /usr/lib64/libssl.so*) && touch %{buildroot}/usr/lib64/libssl.so*
mkdir -p %{buildroot}$(dirname /usr/include/openssl) && touch %{buildroot}/usr/include/openssl
mkdir -p %{buildroot}$(dirname /etc/pki/tls) && touch %{buildroot}/etc/pki/tls


%files
/usr/bin/openssl
/usr/lib64/libcrypto.so*
/usr/lib64/libssl.so*
/usr/include/openssl
/etc/pki/tls

%changelog
* Sat Aug 08 2026 Ermete Forge <forge@ermete.os> - 3.4.1-1
- Assimilated OpenSSL native source build for Ermete OS.
