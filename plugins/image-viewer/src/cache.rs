//! Cache paths for generated display and thumbnail images.

use std::path::{Path, PathBuf};

use omega::{Error, Result};

/// The plugin's cache root. A derivative is keyed by source path, modification
/// time, and length, so a changed file never reuses a stale image.
#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// Resolve the XDG cache directory, `~/.cache/omega/image-viewer`.
    pub fn resolve() -> Result<Self> {
        let base = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
            .unwrap_or_else(|| PathBuf::from(".cache"));
        Self::at(base.join("omega").join("image-viewer"))
    }

    /// Use an explicit cache directory, creating it when absent.
    pub fn at(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// The cache directory in use.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The path of one derivative. `kind` separates display from thumbnail.
    pub fn derivative(&self, source: &Path, kind: &str, size: (u32, u32)) -> Result<PathBuf> {
        let key = self.key(source)?;
        Ok(self.root.join(format!(
            "{key}-{kind}-{}x{}.png",
            size.0.max(1),
            size.1.max(1)
        )))
    }

    /// A stable key for a source file's current contents.
    pub fn key(&self, source: &Path) -> Result<String> {
        let metadata = std::fs::metadata(source).map_err(|error| {
            Error::invalid(format!("cannot inspect {}: {error}", source.display()))
        })?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let mut hash = Fnv1a::new();
        hash.write(source.as_os_str().as_encoded_bytes());
        hash.write_u128(modified);
        hash.write_u64(metadata.len());
        Ok(hash.finish())
    }
}

/// Deterministic FNV-1a, so a cache key never depends on a random hasher seed.
struct Fnv1a(u64);

impl Fnv1a {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    fn write_u128(&mut self, value: u128) {
        self.write(&value.to_le_bytes());
    }

    fn finish(&self) -> String {
        format!("{:016x}", self.0)
    }
}
