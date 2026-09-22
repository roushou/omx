//! Decoding, orientation, and the display/thumbnail derivatives.

use std::path::{Path, PathBuf};

use image::{
    DynamicImage, ImageDecoder, ImageFormat, ImageReader, imageops::FilterType,
    metadata::Orientation,
};
use omega::{Error, Result};

use crate::cache::Cache;

/// A decoded image prepared for one render: the path to display, a thumbnail,
/// and the source's natural size.
#[derive(Debug, Clone)]
pub struct Prepared {
    /// Original file. The renderer applies EXIF orientation when drawing it.
    pub display: PathBuf,
    /// Generated thumbnail path.
    pub thumbnail: PathBuf,
    /// Natural size after orientation is applied.
    pub natural_size: (u32, u32),
    /// Whether the source carried an orientation transform.
    pub rotated: bool,
}

/// A generated thumbnail.
#[derive(Debug, Clone)]
pub struct Thumbnail {
    pub path: PathBuf,
}

/// Decode images once and write bounded derivatives into a [`Cache`].
#[derive(Debug)]
pub struct Deriver;

impl Deriver {
    /// Decode `source`, apply its orientation for the thumbnail, and report the
    /// oriented natural size. The display is always the original file: the
    /// renderer applies EXIF orientation itself, so no display copy is written.
    pub fn prepare(source: &Path, cache: &Cache, thumbnail_edge: u32) -> Result<Prepared> {
        let (mut image, orientation) = Self::decode(source)?;
        image.apply_orientation(orientation);

        let natural_size = (image.width().max(1), image.height().max(1));
        let thumbnail = Self::write(&image, cache, source, "thumb", Self::edge(thumbnail_edge))?;

        Ok(Prepared {
            display: source.to_path_buf(),
            thumbnail,
            natural_size,
            rotated: orientation != Orientation::NoTransforms,
        })
    }

    /// Decode `source` and write a thumbnail no larger than `edge` on its
    /// longest side.
    pub fn thumbnail(source: &Path, cache: &Cache, edge: u32) -> Result<Thumbnail> {
        let (mut image, orientation) = Self::decode(source)?;
        image.apply_orientation(orientation);
        let path = Self::write(&image, cache, source, "thumb", Self::edge(edge))?;
        Ok(Thumbnail { path })
    }

    /// Fit `natural` inside `max`, preserving aspect and never upscaling.
    /// A zero bound yields `(0, 0)` so the caller can draw nothing.
    pub fn fit(natural: (u32, u32), max: (u32, u32)) -> (u32, u32) {
        if max.0 == 0 || max.1 == 0 {
            return (0, 0);
        }
        let width = natural.0.max(1) as f64;
        let height = natural.1.max(1) as f64;
        let scale = (f64::from(max.0) / width)
            .min(f64::from(max.1) / height)
            .min(1.0);
        (
            ((width * scale).round() as u32).max(1),
            ((height * scale).round() as u32).max(1),
        )
    }

    fn decode(source: &Path) -> Result<(DynamicImage, Orientation)> {
        let reader = ImageReader::open(source)
            .and_then(|reader| reader.with_guessed_format())
            .map_err(|error| Self::invalid(source, error))?;
        let mut decoder = reader
            .into_decoder()
            .map_err(|error| Self::invalid(source, error))?;
        let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
        let image =
            DynamicImage::from_decoder(decoder).map_err(|error| Self::invalid(source, error))?;
        Ok((image, orientation))
    }

    fn write(
        image: &DynamicImage,
        cache: &Cache,
        source: &Path,
        kind: &str,
        max: (u32, u32),
    ) -> Result<PathBuf> {
        let fitted = image.resize(max.0.max(1), max.1.max(1), FilterType::Lanczos3);
        let path = cache.derivative(source, kind, (fitted.width(), fitted.height()))?;
        // Two tasks can target one derivative; a unique temp file and a rename
        // keep the published file whole. The cache directory is one filesystem.
        let temp = Self::temp_path(&path);
        if let Err(error) = fitted.save_with_format(&temp, ImageFormat::Png) {
            let _ = std::fs::remove_file(&temp);
            return Err(Self::invalid(source, error));
        }
        std::fs::rename(&temp, &path).map_err(|error| {
            let _ = std::fs::remove_file(&temp);
            Self::invalid(source, error)
        })?;
        Ok(path)
    }

    fn temp_path(path: &Path) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let thread = std::thread::current().id();
        path.with_extension(format!("tmp-{}-{stamp}-{thread:?}", std::process::id()))
    }

    fn edge(edge: u32) -> (u32, u32) {
        let edge = edge.clamp(16, 1024);
        (edge, edge)
    }

    fn invalid(source: &Path, error: impl std::fmt::Display) -> Error {
        Error::invalid(format!("cannot decode {}: {error}", source.display()))
    }
}
