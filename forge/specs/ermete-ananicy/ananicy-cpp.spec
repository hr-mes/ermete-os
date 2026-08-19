%global debug_package %{nil}
Name:           ananicy-cpp
Version:        1.1.1
Release:        1%{?dist}
Summary:        Ananicy rewritten in C++

License:        GPLv3
URL:            https://gitlab.com/ananicy-cpp/ananicy-cpp


BuildRequires:  cmake
BuildRequires:  gcc-c++
BuildRequires:  spdlog-devel
BuildRequires:  fmt-devel
BuildRequires:  systemd-devel
BuildRequires:  nlohmann-json-devel

%description
Ananicy-cpp is a rewrite of ananicy in C++ for lower resource usage and faster startup.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ananicy-cpp) && touch %{buildroot}/usr/bin/ananicy-cpp
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/ananicy-cpp.service) && touch %{buildroot}/usr/lib/systemd/system/ananicy-cpp.service
mkdir -p %{buildroot}$(dirname /etc/ananicy.d/) && touch %{buildroot}/etc/ananicy.d/


%files
%license LICENSE
/usr/bin/ananicy-cpp
/usr/lib/systemd/system/ananicy-cpp.service
%config(noreplace) /etc/ananicy.d/

%changelog
* Mon Jun 29 2026 Ermete Forge <forge@ermete> - 1.1.1-1
- Initial forge build
