# Image viewer

Browse a folder of pictures in a full-window viewer. The plugin lists the
images under a configured path, decodes them in Rust, and draws the selected
picture with a strip of nearby thumbnails.

## Open it

```sh
omega present image-viewer gallery --width 1200 --height 800 \
  --config '{"path":"/home/me/Pictures"}'
```

`--config` carries the folder (or a single image file). `--width` and `--height`
should match the `width` and `height` settings so the picture is fitted to the
window. A repeated invocation reuses the same instance.

Bind it to a compositor shortcut in `~/.config/hypr/bindings.lua`:

```lua
o.bind("SUPER + ALT + P", "Omega pictures", [[
  omega present image-viewer gallery --width 1200 --height 800 \
    --config '{"path":"/home/me/Pictures"}'
]])
```

## Controls

| Input            | Action                    |
| ---------------- | ------------------------- |
| Left / `h`       | Previous image, wrapping  |
| Right / `l`      | Next image, wrapping      |
| Home / End       | First / last image        |
| `+` / `=`        | Zoom in                   |
| `-`              | Zoom out                  |
| `0`              | Fit to the window         |
| Wheel            | Zoom in or out            |
| `r`              | Re-read the folder        |
| Up / Down, click | Move and open a thumbnail |
| Escape           | Close the window          |

The strip keeps the current image centred, so it re-centres as you move through
a large folder instead of scrolling a long list. Magnification runs from 100%
to 800%; when the picture is larger than the window, drag it to pan.

## Settings

| Setting      | Default | Description                                              |
| ------------ | ------- | -------------------------------------------------------- |
| `path`       | empty   | Folder of images or one image file                       |
| `cache`      | empty   | Directory for generated images; empty uses the XDG cache |
| `width`      | `1200`  | Presented window width the picture is fitted to          |
| `height`     | `800`   | Presented window height the picture is fitted to         |
| `thumbnail`  | `72`    | Longest thumbnail side, clamped to 16–256                |
| `max_images` | `500`   | Most directory entries to list, clamped to 1–2000        |

## Behavior

Supported formats are JPEG, PNG, GIF, WebP, BMP, and TIFF. Hidden files,
subdirectories, and other extensions are ignored, and entries are listed by
name.

Images are decoded once in the plugin. The thumbnail is oriented and written
to the cache; the renderer applies EXIF orientation to the original file when
drawing it, so no separate display copy is written. Thumbnails are PNG files
named by a hash of the source path, size, and modification time, so editing a
source never reuses a stale image. The plugin touches only the configured path
and the cache directory; it declares no daemon capabilities.

The strip generates thumbnails a batch at a time as the selection moves, so a
large folder opens without decoding every image up front. A file that fails to
decode shows a placeholder thumbnail and an explanatory message instead of an
empty window.

## Development

```sh
cargo test -p image-viewer
omega preview image-viewer --case gallery
omega preview image-viewer --list
```

The preview cases use the committed `fixtures/` images.
