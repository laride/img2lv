# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-07-15

### Added

- Added JSDoc documentation to all public API functions (`imageToBin`, `imageToC`, `lvglToPng`, `lvglToRgba`, `lvglWidth`, `lvglHeight`) in the generated `index.d.ts`, providing inline descriptions and parameter guidance in editors that support hover documentation.

## [0.2.0] - 2026-06-23

### Added

- Exported `ColorFormat` and `CompressMethod` as napi string enums. Both are available as runtime objects (`ColorFormat.RGB565`) and as TypeScript `const enum` declarations for type-checking.
- `cf` and `compress` now accept string literal union types (`` `${ColorFormat}` `` / `` `${CompressMethod}` ``) — no enum import needed.
- Added `rgb565Dither` option (equivalent to `--rgb565dither` in the Python reference). Enables ordered-dither correction for banding artefacts when converting to RGB565 / RGB565_SWAPPED / RGB565A8 / ARGB8565.
- Added `nemaGfx` option (equivalent to `--nemagfx` in the Python reference). When enabled, palette indices in I8 images are byte-swapped for NEMA GFX accelerator compatibility.

### Changed

- **Breaking:** `imageToBin` and `imageToC` now accept a `ConvertOptions` object as the last argument instead of positional parameters. Callers must migrate to `imageToBin(buf, { cf: 'RGB565' })` style. All fields except `cf` are optional with sensible defaults (`background: 0`, `align: 1`, `premultiply: false`, `compress: 'NONE'`, `rgb565Dither: false`, `nemaGfx: false`).
- **Breaking:** `lvglToPng`, `lvglToRgba`, `lvglWidth`, and `lvglHeight` now accept an optional `DecodeOptions` object (`{ isCArray?: boolean }`) instead of a positional `isCArray: boolean` parameter. Binary callers can drop the second argument entirely; C-array callers must migrate to `{ isCArray: true }`.
- `cf` and `compress` parameters changed from untyped `string` to typed union forms. Existing callers already passing valid string literals require no code changes.

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

[0.2.1]: https://github.com/laride/img2lv/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/laride/img2lv/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/laride/img2lv/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/laride/img2lv/releases/tag/v0.1.0
