# Global UI Overhaul & Settings Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite `ermete-settings-rs` to a Relm4 component architecture using a global CSS design system, and implement DBus live theming sync.

**Architecture:** Convert `ermete-settings-rs` to use Relm4 with an `AppModel` and a modular page component system. A central CSS file will provide the glassmorphism and premium tokens.

**Tech Stack:** Rust, GTK4, Relm4, zbus, Tokio.

## Global Constraints

- Must compile with `cargo build`.
- Code must reside in `specs/ermete-settings-rs/ermete-settings-rs-1.0.0/`.
- Use idiomatic Relm4 component structures.

---

### Task 1: Add Relm4 Dependency and Setup Root CSS

**Files:**
- Modify: `specs/ermete-settings-rs/ermete-settings-rs-1.0.0/Cargo.toml`
- Create: `specs/ermete-settings-rs/ermete-settings-rs-1.0.0/src/style.rs`

**Interfaces:**
- Consumes: None
- Produces: `style::load_global_css()`

- [ ] **Step 1: Update Cargo.toml**
Add `relm4 = "0.7"` to dependencies.

- [ ] **Step 2: Create CSS Loader**
Create `src/style.rs` with the `load_global_css()` function that defines glassmorphism tokens.

- [ ] **Step 3: Commit**
```bash
git add Cargo.toml src/style.rs
git commit -m "chore: add relm4 and global css loader"
```

### Task 2: Scaffold Relm4 AppModel

**Files:**
- Modify: `specs/ermete-settings-rs/ermete-settings-rs-1.0.0/src/main.rs`

**Interfaces:**
- Consumes: `style::load_global_css()`
- Produces: Basic Relm4 Application loop.

- [ ] **Step 1: Rewrite main.rs for Relm4**
Replace GTK4 manual loop with Relm4 `RelmApp::new("os.ermete.Settings").run::<AppModel>(())`.

- [ ] **Step 2: Verify Compilation**
Run `cargo check` to ensure the boilerplate compiles.

- [ ] **Step 3: Commit**
```bash
git add src/main.rs
git commit -m "refactor: migrate root app to relm4"
```
