use landlock::{
    AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr, ABI,
};
use std::env;
use std::path::Path;

/// Applica la policy di sandboxing Landlock al processo corrente.
/// Concede accesso in sola lettura a `/usr`, `/etc` e `$XDG_RUNTIME_DIR`.
/// Neghi qualsiasi accesso ad altre directory (deny-by-default).
pub fn apply_landlock_sandbox() -> Result<(), Box<dyn std::error::Error>> {
    let abi = ABI::V1;
    let read_access = AccessFs::from_read(abi);

    let mut ruleset = Ruleset::default()
        .handle_access(read_access)?
        .create()?;

    let mut paths_to_allow = vec!["/usr".to_string(), "/etc".to_string()];

    if let Ok(xdg_runtime) = env::var("XDG_RUNTIME_DIR") {
        if !xdg_runtime.is_empty() && Path::new(&xdg_runtime).exists() {
            paths_to_allow.push(xdg_runtime);
        }
    }

    for path in paths_to_allow {
        if Path::new(&path).exists() {
            if let Ok(path_fd) = PathFd::new(&path) {
                ruleset = ruleset.add_rule(PathBeneath::new(path_fd, read_access))?;
            }
        }
    }

    ruleset.restrict_self()?;
    Ok(())
}
