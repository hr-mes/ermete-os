Name:           sccache
Version:        0.9.1
Release:        1%{?dist}
Summary:        Ermete OS Core Component - sccache

License:        GPLv3
URL:            https://github.com/mozilla/sccache
Source0:        https://github.com/mozilla/sccache/archive/refs/tags/v%{version}.tar.gz#/sccache-%{version}.tar.gz

%description
Core component implementation for sccache.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release

%install
mkdir -p %{buildroot}/usr/bin
install -Dm755 target/release/sccache %{buildroot}/usr/bin/sccache

%files
/usr/bin/sccache

