//! Discovering the image files under a configured path.

use std::path::{Path, PathBuf};

use omega::{Error, Result};

/// One image discovered under the configured path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageFile {
    /// Absolute, canonical path so the renderer can load it.
    pub path: PathBuf,
    /// Final path component, used as the display name.
    pub name: String,
}

/// Image file discovery. Every operation hangs off this type.
#[derive(Debug)]
pub struct Catalog;

impl Catalog {
    /// Extensions this viewer decodes, lower case and without the dot.
    pub const EXTENSIONS: &'static [&'static str] =
        &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tif", "tiff"];

    /// List the images at `path`, in name order.
    ///
    /// A file path yields one entry when its extension is supported. A directory
    /// yields its supported, non-hidden regular files, bounded to `limit`.
    /// Anything else is refused with a message naming the path.
    pub fn discover(path: &Path, limit: usize) -> Result<Vec<ImageFile>> {
        let metadata = std::fs::metadata(path)
            .map_err(|error| Error::invalid(format!("cannot read {}: {error}", path.display())))?;

        if metadata.is_file() {
            return if Self::supports(path) {
                Self::resolve(path).map(|file| vec![file])
            } else {
                Err(Error::invalid(format!(
                    "{} is not a supported image",
                    path.display()
                )))
            };
        }

        if !metadata.is_dir() {
            return Err(Error::invalid(format!(
                "{} is neither a file nor a directory",
                path.display()
            )));
        }

        let entries = std::fs::read_dir(path)
            .map_err(|error| Error::invalid(format!("cannot list {}: {error}", path.display())))?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| {
                Error::invalid(format!("cannot list {}: {error}", path.display()))
            })?;
            let candidate = entry.path();
            if Self::is_hidden(&candidate) || !Self::is_regular_file(&candidate)? {
                continue;
            }
            if Self::supports(&candidate) {
                files.push(candidate);
            }
        }

        files.sort_by(|left, right| Self::sort_name(left).cmp(&Self::sort_name(right)));
        files.truncate(limit.max(1));

        files.into_iter().map(|path| Self::resolve(&path)).collect()
    }

    /// Whether the path's extension is one this viewer decodes.
    pub fn supports(path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .is_some_and(|extension| Self::EXTENSIONS.contains(&extension.as_str()))
    }

    /// Hidden entries are metadata, not pictures; keep dotfiles out of the strip.
    fn is_hidden(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'))
    }

    fn is_regular_file(path: &Path) -> Result<bool> {
        match std::fs::metadata(path) {
            Ok(metadata) => Ok(metadata.is_file()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(Error::invalid(format!(
                "cannot inspect {}: {error}",
                path.display()
            ))),
        }
    }

    fn sort_name(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
    }

    /// Resolve one path to its absolute form and display name.
    fn resolve(path: &Path) -> Result<ImageFile> {
        let absolute = std::fs::canonicalize(path).map_err(|error| {
            Error::invalid(format!("cannot resolve {}: {error}", path.display()))
        })?;
        let name = absolute
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| absolute.to_string_lossy().into_owned());
        Ok(ImageFile {
            path: absolute,
            name,
        })
    }
}
