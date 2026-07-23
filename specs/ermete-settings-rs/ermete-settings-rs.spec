%global debug_package %{nil}
Name:           ermete-settings-rs
Version:        1.0.0
Release:        8%{?dist}
Summary:        Pure Rust native System Settings for Ermete OS

License:        GPLv3+
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo
BuildRequires:  gtk4-devel

%description
Ermete Settings is the native control panel for Ermete OS, written in pure Rust with GTK4.

%prep
%autosetup

%build
%set_build_flags
%cargo_build

%install
rm -rf $RPM_BUILD_ROOT
install -d $RPM_BUILD_ROOT/%{_bindir}
install -m 0755 target/release/%{name} $RPM_BUILD_ROOT/%{_bindir}/%{name}
install -d $RPM_BUILD_ROOT/%{_datadir}/applications
install -m 0644 os.ermete.Settings.desktop $RPM_BUILD_ROOT/%{_datadir}/applications/os.ermete.Settings.desktop

%files
%{_bindir}/%{name}
%{_datadir}/applications/os.ermete.Settings.desktop

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-8
- Implemented Enterprise Wi-Fi (802.1x EAP-TLS/PEAP) and Native VPN tunnel management (WireGuard/OpenVPN) in network page
- Added Flatpak Sandbox Permissions Manager (`--socket=wayland/pulseaudio`, `--share=network`, `--filesystem=home`) in privacy page
- Added Focus Modes (Do Not Disturb, Gaming Mode, Work Session, Reading) and Niri automation rules in focus page

* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-7
- Implement Phase 4c: native KDL updater without subprocesses, VRR/HDR switches, display spatial layout preview, and trackpad gestures
- Eliminate CLI subprocess calls (wpctl, nmcli, uname, whoami, niri) and integrate native Niri UNIX socket client (`niri_client.rs`), D-Bus `os.ermete.Bedrock` proxy, and sysfs/procfs readers.

* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 1.0.0-3
- Dynamic Matugen theme generation & wallpaper integration

* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 1.0.0-2
- Complete audio sync, desktop entry and bump spec release to 2

