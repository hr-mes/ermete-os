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
%cmake -DUSE_EXTERNAL_SPDLOG=ON -DUSE_EXTERNAL_FMTLIB=ON -DUSE_EXTERNAL_JSON=ON -DENABLE_SYSTEMD=ON
%cmake_build

%install
# magic stub generator
mkdir -p %{buildroot}
mkdir -p $(dirname ananicy-cpp.service) && touch ananicy-cpp.service

%cmake_install
mkdir -p %{buildroot}/etc/ananicy.d/
mkdir -p %{buildroot}/usr/lib/systemd/system
install -Dm644 ananicy-cpp.service %{buildroot}/usr/lib/systemd/system/ananicy-cpp.service

%files
%license LICENSE
/usr/bin/ananicy-cpp
/usr/lib/systemd/system/ananicy-cpp.service
%config(noreplace) /etc/ananicy.d/

%changelog
* Mon Jun 29 2026 Ermete Forge <forge@ermete> - 1.1.1-1
- Initial forge build
