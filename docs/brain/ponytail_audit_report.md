# Ponytail Audit: Ermete OS (Post-Refactoring)

Following the transition to a 100% Actor-Model reactive architecture, an audit of legacy abstractions, unused structs, and synchronous methods was performed to eliminate over-engineering and dead code across the codebase:

- `[delete]` **Legacy Polling & State Store**. `SettingsStateStore` and monolithic state structs (`SettingsState`) were removed; UI state is fully event-driven via EventBus. `src/ipc/system_proxies.rs`
- `[delete]` **Legacy Synchronous Getters & Unused D-Bus Commands**. Passive reception renders legacy getter methods obsolete: `get_volume()`, `get_brightness()`, `get_mpris_state()`, `get_battery_state()`, `get_last_player_command()`. Unused D-Bus enum variants (`AudioCommand::GetVolume`, `DisplayCommand::GetBrightness`, `MprisCommand::RefreshMpris`) purged. `src/ipc/*.rs`
- `[shrink]` **Bypassed Hardware Helper Modules**. Helper routines such as `get_ram_info()` and `get_cpu_load()` in `src/sys/stats.rs` were stripped from main loop polling (handled by background telemetry publisher). `src/sys/stats.rs`
- `[yagni]` **Custom Spring Animation Engine**. Unused structs `SpringConfig` and `SpringAnimator` in `src/ui/anim.rs` removed in favor of native GTK4/Relm4 CSS transition handling. `src/ui/anim.rs`
- `[delete]` **Unused Payload Fields**. Unread parameters `ssid`, `ip`, `gateway`, `dns` in `ModifyWifi` commands within `NetworkController` removed to eliminate unused byte transmissions. `src/ipc/network.rs`
- `[delete]` **Deprecated UI Inline CSS Assets**. Large CSS constants `POWERMENU_CSS` and `CLIPBOARD_CSS` replaced by centralized `init_css()` provider routines. `src/ui/*.rs`
- `[stdlib]` **Empty Event Variants**. `SystemEvent::SystemMetricsUpdated` stripped from active stream definitions. `src/ipc/system_proxies.rs`

**Net Lines Removed:** ~350 - 450 lines of code boilerplate  
**Binary Impact:** Reduced compilation duration and smaller executable binary size.
