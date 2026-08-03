use std::env;
use std::path::Path;

/// RAII Guard for temporarily controlling the `HOME` environment variable in tests and auth flows.
#[derive(Debug)]
pub struct HomeGuard {
    original: Option<String>,
}

impl HomeGuard {
    pub fn set(new_home: &Path) -> Self {
        let original = env::var("HOME").ok();
        env::set_var("HOME", new_home.to_str().unwrap_or("/tmp"));
        Self { original }
    }
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(val) => env::set_var("HOME", val),
            None => env::remove_var("HOME"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_guard_restoration() {
        let orig = env::var("HOME").ok();
        {
            let temp_dir = std::env::temp_dir().join("ermete_home_guard_test");
            let _guard = HomeGuard::set(&temp_dir);
            assert_eq!(env::var("HOME").unwrap(), temp_dir.to_str().unwrap());
        }
        assert_eq!(env::var("HOME").ok(), orig);
    }
}
