use std::path::Path;

use serde::de::DeserializeOwned;

use crate::{SanthError, SanthResult};

/// Resolve and load a Tier A TOML config.
///
/// Precedence is `path` first, then each `default_paths` entry in order, then
/// `T::default()` when no candidate exists.
pub fn resolve_config<T>(path: Option<&Path>, default_paths: &[&Path]) -> SanthResult<T>
where
    T: DeserializeOwned + Default,
{
    if let Some(path) = path {
        return load_config(path);
    }

    for candidate in default_paths {
        // `Path::is_file()` collapses every error (permission denied, ENOTDIR,
        // ...) to `false`, silently defaulting instead of surfacing a real IO
        // problem. Inspect the metadata error kind: only a genuine NotFound may
        // fall through to the next candidate / default; anything else is a read
        // failure the operator must see.
        match std::fs::metadata(candidate) {
            Ok(meta) if meta.is_file() => return load_config(candidate),
            Ok(_) => {} // exists but is not a regular file (e.g. a directory)
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(SanthError::ConfigRead {
                    path: candidate.to_path_buf(),
                    source,
                });
            }
        }
    }

    Ok(T::default())
}

fn load_config<T>(path: &Path) -> SanthResult<T>
where
    T: DeserializeOwned,
{
    let contents = std::fs::read_to_string(path).map_err(|source| SanthError::ConfigRead {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&contents).map_err(|source| SanthError::ConfigParse {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn resolve_config_surfaces_non_notfound_io_error_on_default_path() {
        // Create a regular file, then use a default path that treats that file
        // as a directory component. `metadata()` then fails with ENOTDIR (a
        // NON-NotFound error), which must surface as ConfigRead instead of
        // silently falling back to `T::default()` as the old `is_file()` did.
        let base = std::env::temp_dir().join("santh-cli-config-enotdir");
        let _ = std::fs::create_dir_all(&base);
        let file = base.join("not-a-dir");
        std::fs::write(&file, b"x").unwrap();
        let bogus = file.join("config.toml"); // a regular file used as a directory
        let result: SanthResult<HashMap<String, String>> =
            resolve_config(None, &[bogus.as_path()]);
        assert!(
            matches!(result, Err(SanthError::ConfigRead { .. })),
            "a non-NotFound IO error on a default path must surface as ConfigRead"
        );
    }

    #[test]
    fn resolve_config_defaults_when_default_path_is_absent() {
        // A genuinely absent default path (NotFound) must still fall through to
        // the default config — the fix must not turn NotFound into an error.
        let absent =
            std::env::temp_dir().join("santh-cli-config-definitely-absent-xyz.toml");
        let _ = std::fs::remove_file(&absent);
        let result: SanthResult<HashMap<String, String>> =
            resolve_config(None, &[absent.as_path()]);
        assert!(result.is_ok(), "an absent default path must default, not error");
        assert!(result.unwrap().is_empty());
    }
}
