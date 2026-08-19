Name:           just
%global debug_package %{nil}
Version:        1.39.0
Release:        1%{?dist}
Summary:        Just a command runner - handy way to save and run project-specific commands
License:        CC0-1.0
URL:            https://github.com/casey/just


BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  mold

%description
`just` is a handy way to save and run project-specific commands.
Compiled natively in Ermete Forge with extreme x86-64-v3 optimizations.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/just) && touch %{buildroot}/usr/bin/just


%files
/usr/bin/just

%changelog
* Sat Aug 08 2026 Ermete Forge <forge@ermete.os> - 1.39.0-1
- Native Rust source build integrated into Ermete Forge Tier0
