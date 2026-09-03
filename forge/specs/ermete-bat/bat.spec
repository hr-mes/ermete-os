Name:           bat
Version:        0.26.1
Release:        1%{?dist}
Summary:        Cat(1) clone with syntax highlighting and Git integration

License:        MIT OR Apache-2.0
URL:            https://github.com/sharkdp/bat
Source0:        https://github.com/sharkdp/bat/archive/refs/tags/v%{version}.tar.gz#/bat-%{version}.tar.gz

BuildRequires:  cargo rust

%description
A cat(1) clone with syntax highlighting, Git integration and automatic paging,
built from the upstream release with the crates pinned by its Cargo.lock.

%prep
%autosetup -n bat-%{version}

%build
%set_build_flags
# build.rs genera pagina man e completamenti in BAT_ASSETS_GEN_DIR (altrimenti in OUT_DIR).
export BAT_ASSETS_GEN_DIR="$PWD/assets-gen"
cargo build --release --locked

%install
install -D -m 0755 target/release/bat %{buildroot}%{_bindir}/bat
install -D -m 0644 assets-gen/assets/manual/bat.1 %{buildroot}%{_mandir}/man1/bat.1
install -D -m 0644 assets-gen/assets/completions/bat.bash %{buildroot}%{_datadir}/bash-completion/completions/bat
install -D -m 0644 assets-gen/assets/completions/bat.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/bat.fish
install -D -m 0644 assets-gen/assets/completions/bat.zsh %{buildroot}%{_datadir}/zsh/site-functions/_bat

%files
%license LICENSE-APACHE LICENSE-MIT
%{_bindir}/bat
%{_mandir}/man1/bat.1*
%{_datadir}/bash-completion/completions/bat
%{_datadir}/fish/vendor_completions.d/bat.fish
%{_datadir}/zsh/site-functions/_bat

%changelog
* Thu Sep 03 2026 Ermete Forge <forge@ermete.os> - 0.26.1-1
- Spec riscritta per il builder Ermete: Source0 dall'archivio upstream verificato
  da SOURCES/sources.sha256, cargo build --locked, niente macro cargo-rpm-macros
  di Fedora né patch di spacchettamento dei crate
