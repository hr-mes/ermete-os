# Scope: Define Rust-UI

## Architecture
- `Rust-UI`: Wayland/GTK4 Rust stack (`ermete-shell-rs`, `ermete-settings-rs`, Niri IPC).

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Define Rust-UI | Create domain skill / system prompt for Rust-UI in `.agents/skills/ermete-rust-ui` | none | IN_PROGRESS |

## Interface Contracts
### Rust-UI ↔ All Agents
- Rust-UI must delegate out-of-scope tasks (like system dependencies) by explicitly identifying the required domain (e.g. Forge-Builder) and forwarding the request.
