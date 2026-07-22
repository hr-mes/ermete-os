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

