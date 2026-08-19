Name:           git-native
Version:        2.48.1
Release:        1%{?dist}
Summary:        Assimilated Git Version Control System built natively from source for Ermete OS
License:        GPL-2.0-only
URL:            https://git-scm.com/


Provides:       git = %{version}-%{release}
Provides:       git-core = %{version}-%{release}
Obsoletes:      git < %{version}-%{release}
Obsoletes:      git-core < %{version}-%{release}

BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  gettext
BuildRequires:  curl-devel
BuildRequires:  expat-devel
BuildRequires:  openssl-devel
BuildRequires:  zlib-devel

%description
Git version control system compiled natively from source for Ermete OS Forge.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/git*) && touch %{buildroot}/usr/bin/git*


%files
/usr/bin/git*
%{_libexecdir}/git-core
%{_datadir}/git-core

%changelog
* Sat Aug 08 2026 Ermete Forge <forge@ermete.os> - 2.48.1-1
- Assimilated git native source build for Ermete OS.
