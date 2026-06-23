# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-06-23

### Added

- Added an indexed PNG fast path for `AUTO` and explicit indexed formats so palette entries and packed indices can be preserved without requantization when the target LVGL tier has enough capacity.
- Added the `OPTIMIZED` color request mode, which can rewrite indexed PNG palettes to a smaller LVGL indexed tier when that reduces output size without changing the visible result.
- Added generated-C-array decoding coverage for `lvglWidth()`, `lvglHeight()`, `lvglToPng()`, and `lvglToRgba()` when the source was produced by `img2lv`.

### Changed

- `AUTO` now preserves source indexed PNG palette sizing and index data to stay closer to the Python reference behavior for indexed inputs.
- Fully transparent pixels are normalized during indexed analysis so invisible RGB differences do not consume separate palette entries.
- Premultiplied decode paths now un-premultiply back to straight-alpha RGBA/PNG output, including `ARGB8888_PREMULTIPLIED`.
- Expanded the README to document `AUTO` vs `OPTIMIZED`, C-array parsing limits, LVGL header size constraints, lint guidance, and known reference-compatibility differences.

### Fixed

- Reduced unexpected visual differences when converting indexed PNG assets by avoiding unnecessary palette churn and requantization.
- Improved palette stability for images containing multiple fully transparent pixels with different RGB payloads.
- Added validation for image dimensions, aligned stride size, and unsupported premultiply/color-format combinations before emitting invalid LVGL output.
- Improved trailing-repeat handling in RLE compression so repeat runs can be emitted as their own compact blocks.

## [0.1.0] - 2026-06-15

### Added

- Initial public release of `img2lv` with Node.js bindings for converting common static images to and from LVGL v9 image data.

[0.1.1]: https://github.com/laride/img2lv/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/laride/img2lv/releases/tag/v0.1.0
