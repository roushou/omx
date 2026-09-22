//! Catalogue, derivative, and surface behavior.

use std::path::Path;

use image::{Rgb, RgbImage};
use omega::config::Fields;
use omega::testing::{Drawn, State, SurfaceHarness};
use tempfile::TempDir;

use super::{Cache, Catalog, Deriver, Message, Settings, Viewer};

fn write_png(path: &Path, width: u32, height: u32, color: [u8; 3]) {
    RgbImage::from_pixel(width, height, Rgb(color))
        .save(path)
        .expect("a writable fixture");
}

fn settings(images: &TempDir, cache: &TempDir) -> Settings {
    Settings {
        path: images.path().to_string_lossy().into_owned(),
        cache: cache.path().to_string_lossy().into_owned(),
        width: 640,
        height: 480,
        thumbnail: 48,
        max_images: 100,
    }
}

#[test]
fn the_manifest_declares_the_gallery_surface() {
    let manifest = omega::testing::manifest_of(&super::plugin());
    assert_eq!(manifest.surfaces.len(), 1);
    assert_eq!(manifest.surfaces[0].id, "gallery");
}

#[test]
fn the_catalogue_keeps_supported_nonhidden_regular_files_in_name_order() {
    let images = TempDir::new().unwrap();
    write_png(&images.path().join("b.PNG"), 4, 4, [0, 0, 255]);
    write_png(&images.path().join("a.jpg"), 4, 4, [255, 0, 0]);
    std::fs::write(images.path().join("notes.txt"), b"nope").unwrap();
    write_png(&images.path().join(".hidden.png"), 4, 4, [0, 255, 0]);
    std::fs::create_dir(images.path().join("nested.png")).unwrap();

    let files = Catalog::discover(images.path(), 100).unwrap();
    let names: Vec<_> = files.iter().map(|file| file.name.as_str()).collect();
    assert_eq!(names, ["a.jpg", "b.PNG"]);
    assert!(files.iter().all(|file| file.path.is_absolute()));
}

#[test]
fn a_single_supported_file_is_one_entry() {
    let images = TempDir::new().unwrap();
    let file = images.path().join("only.png");
    write_png(&file, 4, 4, [0, 0, 0]);

    let files = Catalog::discover(&file, 10).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "only.png");
}

#[test]
fn an_unsupported_or_missing_path_is_refused() {
    let images = TempDir::new().unwrap();
    let text = images.path().join("notes.txt");
    std::fs::write(&text, b"nope").unwrap();

    assert!(Catalog::discover(&text, 10).is_err());
    assert!(Catalog::discover(&images.path().join("missing"), 10).is_err());
}

#[test]
fn fitting_preserves_aspect_and_never_upscales() {
    assert_eq!(Deriver::fit((400, 200), (200, 200)), (200, 100));
    assert_eq!(Deriver::fit((100, 100), (400, 400)), (100, 100));
    assert_eq!(Deriver::fit((100, 100), (0, 100)), (0, 0));
}

#[test]
fn preparing_an_upright_image_keeps_the_original_display_and_writes_a_thumbnail() {
    let images = TempDir::new().unwrap();
    let cached = TempDir::new().unwrap();
    let file = images.path().join("photo.png");
    write_png(&file, 200, 100, [10, 20, 30]);
    let cache = Cache::at(cached.path()).unwrap();

    let prepared = Deriver::prepare(&file, &cache, 32).unwrap();

    assert_eq!(prepared.display, file);
    assert!(!prepared.rotated);
    assert_eq!(prepared.natural_size, (200, 100));
    assert!(prepared.thumbnail.exists());
    let (width, height) = image::image_dimensions(&prepared.thumbnail).unwrap();
    assert!(
        width <= 32 && height <= 32,
        "thumbnail was {width}×{height}"
    );
}

#[test]
fn concurrent_derivatives_leave_a_readable_file() {
    let images = TempDir::new().unwrap();
    let cached = TempDir::new().unwrap();
    let file = images.path().join("photo.png");
    write_png(&file, 200, 100, [40, 80, 120]);
    let cache = Cache::at(cached.path()).unwrap();

    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| Deriver::thumbnail(&file, &cache, 32).unwrap());
        }
    });

    let thumbnail = Deriver::thumbnail(&file, &cache, 32).unwrap();
    assert!(image::image_dimensions(&thumbnail.path).is_ok());
}

#[tokio::test]
async fn the_viewer_lists_images_and_prepares_the_selected_one() {
    let images = TempDir::new().unwrap();
    let cached = TempDir::new().unwrap();
    write_png(&images.path().join("a.png"), 80, 40, [255, 0, 0]);
    write_png(&images.path().join("b.png"), 40, 80, [0, 0, 255]);

    let mut viewer =
        SurfaceHarness::<Viewer>::configured(&State::new(), &settings(&images, &cached).write())
            .unwrap();

    // Catalogue, then the prepare and one strip thumbnail run together.
    viewer.complete().await.unwrap();
    viewer.complete().await.unwrap();
    viewer.complete().await.unwrap();

    let drawn = viewer.draw();
    assert!(drawn.text().contains("a.png"));
    assert!(drawn.text().contains("1 / 2"));
    assert!(drawn.text().contains("80 × 40"));
    let picture = drawn.prop("picture", "source").expect("a selected picture");
    assert!(Path::new(&picture).is_absolute());
    assert!(
        drawn
            .kinds()
            .iter()
            .filter(|kind| **kind == "image")
            .count()
            >= 2
    );
}

#[tokio::test]
async fn stepping_wraps_and_switches_the_picture() {
    let images = TempDir::new().unwrap();
    let cached = TempDir::new().unwrap();
    write_png(&images.path().join("a.png"), 80, 40, [255, 0, 0]);
    write_png(&images.path().join("b.png"), 40, 80, [0, 0, 255]);

    let mut viewer =
        SurfaceHarness::<Viewer>::configured(&State::new(), &settings(&images, &cached).write())
            .unwrap();
    viewer.complete().await.unwrap();
    viewer.complete().await.unwrap();
    viewer.complete().await.unwrap();
    let first = viewer.draw().prop("picture", "source").unwrap();

    viewer.send(Message::Step(1)).unwrap();
    viewer.complete().await.unwrap();

    let drawn = viewer.draw();
    assert!(drawn.text().contains("2 / 2"));
    assert_ne!(first, drawn.prop("picture", "source").unwrap());

    viewer.send(Message::Step(1)).unwrap();
    viewer.complete().await.unwrap();
    assert!(viewer.draw().text().contains("1 / 2"), "stepping wraps");
}

#[tokio::test]
async fn an_empty_path_shows_the_open_instructions() {
    let cached = TempDir::new().unwrap();
    let images = TempDir::new().unwrap();
    let settings = Settings {
        path: String::new(),
        ..settings(&images, &cached)
    };

    let drawn = Drawn::configured::<Viewer>(&State::new(), &settings.write()).unwrap();
    let status = drawn.first("status").expect("a status node");
    assert_eq!(
        drawn.prop(&status, "title").as_deref(),
        Some("No folder chosen")
    );
}

#[tokio::test]
async fn an_unreadable_path_is_reported() {
    let cached = TempDir::new().unwrap();
    let images = TempDir::new().unwrap();
    let settings = Settings {
        path: "/definitely/not/a/real/path".into(),
        ..settings(&images, &cached)
    };

    let mut viewer =
        SurfaceHarness::<Viewer>::configured(&State::new(), &settings.write()).unwrap();
    viewer.complete().await.unwrap();

    let drawn = viewer.draw();
    let status = drawn.first("status").expect("a status node");
    assert_eq!(
        drawn.prop(&status, "title").as_deref(),
        Some("Nothing to show")
    );
    assert!(
        drawn
            .prop(&status, "message")
            .is_some_and(|message| message.contains("cannot read"))
    );
}
