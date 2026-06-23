# img2lv

Convert common static images (PNG, JPEG, WebP, BMP, etc.) to and from LVGL v9 image data.

Supports Node.js and Browser (via WebAssembly). Compatibility with other JS runtimes such as Deno and Bun has not been tested.

## Installation

```bash
npm install img2lv
# or
yarn add img2lv
# or
pnpm add img2lv
```

The package ships prebuilt native binaries for major platforms. In environments where a native binary is unavailable, it automatically falls back to a WASI-based WebAssembly build.

## Usage

### Node.js

```js
import { readFileSync, writeFileSync } from 'node:fs'
import { imageToBin, imageToC, lvglToPng } from 'img2lv'

const png = readFileSync('icon.png')

// Convert to LVGL binary format — only `cf` is required, everything else is optional
const bin = imageToBin(png, { cf: 'ARGB8888' })
writeFileSync('icon.bin', bin)

// Convert to LVGL C source file with LZ4 compression
const cSource = imageToC(png, 'icon.png', null, { cf: 'RGB565', compress: 'LZ4' })
writeFileSync('icon.c', cSource)

// Convert LVGL binary back to PNG
const pngOut = lvglToPng(bin)
writeFileSync('icon_out.png', pngOut)
```

### Browser

The browser entry point loads the WASM build automatically:

```js
import { imageToBin } from 'img2lv'

const file = await fetch('/icon.png').then((r) => r.arrayBuffer())
const bin = imageToBin(Buffer.from(file), { cf: 'ARGB8888' })
```

> **Note:** C array parsing (`isCArray = true`) is not available in the browser build.

## API

### `imageToBin(input, options)`

Convert an encoded image to LVGL binary format.

| Parameter              | Type      | Description                                                                        |
| ---------------------- | --------- | ---------------------------------------------------------------------------------- |
| `input`                | `Buffer`  | Encoded image bytes (PNG, JPEG, WebP, BMP, etc.)                                   |
| `options.cf`           | `string`  | **Required.** Color format (see table below), or `'AUTO'` / `'OPTIMIZED'`          |
| `options.background`   | `number`  | Background color as 24-bit RGB (e.g. `0xFF0000`). Default: `0`                     |
| `options.align`        | `number`  | Stride alignment in bytes (any positive integer). Default: `1`                     |
| `options.premultiply`  | `boolean` | Pre-multiply RGB channels with alpha. Default: `false`                             |
| `options.compress`     | `string`  | `'NONE'`, `'RLE'`, or `'LZ4'`. Default: `'NONE'`                                   |
| `options.rgb565Dither` | `boolean` | Enable ordered dithering for RGB565 variants. Default: `false`                     |
| `options.nemaGfx`      | `boolean` | Byte-swap palette indices for NEMA GFX accelerator compatibility. Default: `false` |

**Returns:** `Buffer` — LVGL binary data (12-byte header + pixel data)

Input constraints:

- Image width and height must both be at least `1`.
- LVGL stores `w`, `h`, and `stride` in 16-bit header fields. In practice this means width, height, and the final aligned stride must all fit within `65535`.
- Very wide images may still be rejected even when `width <= 65535`, because higher-bpp formats and large `align` values can push the computed stride past the LVGL header limit.

### `imageToC(input, inputName, outputName, options)`

Convert an encoded image to a LVGL C source file.

| Parameter    | Type             | Description                                               |
| ------------ | ---------------- | --------------------------------------------------------- |
| `input`      | `Buffer`         | Encoded image bytes                                       |
| `inputName`  | `string`         | Original filename (used for variable naming)              |
| `outputName` | `string \| null` | Override variable name, or `null` to derive from filename |
| `options`    | `ConvertOptions` | Same options as `imageToBin`                              |

**Returns:** `string` — C source code containing the `lv_image_dsc_t` descriptor

The same dimension and stride limits from `imageToBin()` apply here.

### `lvglToPng(input, options?)`

Convert LVGL image data back to PNG.

| Parameter          | Type      | Description                                                    |
| ------------------ | --------- | -------------------------------------------------------------- |
| `input`            | `Buffer`  | LVGL binary data, or UTF-8 encoded C source                    |
| `options.isCArray` | `boolean` | `true` if input is a C array from `imageToC`. Default: `false` |

**Returns:** `Buffer` — PNG file bytes

> C array parsing is intended for testing and debugging only and should not be relied on in production.

### `lvglToRgba(input, options?)`

Decode LVGL image data to raw RGBA pixels.

| Parameter          | Type      | Description                                                    |
| ------------------ | --------- | -------------------------------------------------------------- |
| `input`            | `Buffer`  | LVGL binary data, or UTF-8 encoded C source                    |
| `options.isCArray` | `boolean` | `true` if input is a C array from `imageToC`. Default: `false` |

**Returns:** `Buffer` — Raw RGBA pixel data (4 bytes per pixel, row-major)

### `lvglWidth(input, options?)` / `lvglHeight(input, options?)`

Get image dimensions from LVGL data. Accept the same `options` as `lvglToPng`.

**Returns:** `number`

## Supported Color Formats

| Format                    | Description                                                                            |
| ------------------------- | -------------------------------------------------------------------------------------- |
| `I1` / `I2` / `I4` / `I8` | Indexed with 2 / 4 / 16 / 256 color palette                                            |
| `A1` / `A2` / `A4` / `A8` | Alpha-only                                                                             |
| `AL88`                    | 8-bit luminance + 8-bit alpha                                                          |
| `L8`                      | 8-bit grayscale                                                                        |
| `RGB565`                  | 16-bit RGB (little-endian)                                                             |
| `RGB565_SWAPPED`          | 16-bit RGB (big-endian)                                                                |
| `RGB565A8`                | 16-bit RGB + separate alpha plane                                                      |
| `ARGB8565`                | 16-bit RGB + inline alpha byte                                                         |
| `RGB888`                  | 24-bit RGB                                                                             |
| `ARGB8888`                | 32-bit RGBA                                                                            |
| `XRGB8888`                | 32-bit RGB (alpha forced to 0xFF)                                                      |
| `ARGB8888_PREMULTIPLIED`  | 32-bit premultiplied RGBA                                                              |
| `AUTO`                    | Match the Python reference behavior for automatic indexed selection                    |
| `OPTIMIZED`               | Automatically compress indexed palettes when that produces a smaller LVGL indexed tier |

### AUTO vs OPTIMIZED

- `AUTO` is intended to stay compatible with the Python reference converter. In particular, for source indexed PNG files it preserves the original palette/index stream and selects the LVGL indexed tier from the source palette size.
- `OPTIMIZED` is specific to this Rust/JS project and does not exist in the Python reference converter.
- `OPTIMIZED` may rewrite indexed palettes when that can reduce the LVGL output size. If the resulting indexed tier is not smaller than the original one, the original palette is kept.

## Supported Platforms

Prebuilt native binaries are provided for:

- **Windows** — x64, ARM64
- **macOS** — x64, ARM64 (Apple Silicon)
- **Linux** — x64 (glibc / musl), ARM64 (glibc / musl)
- **FreeBSD** — x64

A **WebAssembly** (WASI) fallback is included for all other environments, including browsers.

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) >= 18
- [Yarn](https://yarnpkg.com/) 4.x

### Development

```bash
# Install dependencies
yarn install

# Build native addon (debug)
yarn build:debug

# Build native addon (release)
yarn build

# Run tests
yarn test

# Format code
yarn format

# Lint
yarn lint # JS
cargo clippy # Rust
```

> **Tip:** Before submitting a PR, make sure to run at least one build, format the code, and run both the JS and Rust lint checks.

### Cross-compilation

To build for a specific target:

```bash
yarn build --target aarch64-apple-darwin
yarn build --target x86_64-unknown-linux-musl -x
yarn build --target wasm32-wasip1-threads
```

For musl targets, [Zig](https://ziglang.org/) and `cargo-zigbuild` are required.

## Reference Compatibility

This project's conversion logic is based on the official LVGL Python conversion script, which can be found in the `reference/` directory.
Most behaviors are intentionally kept compatible with that version, but there are also a few implementation differences in this project:

- `AUTO` for non-indexed inputs follows the Rust implementation's own palette analysis path instead of the Python script's `pngquant`-based conversion flow.
- `OPTIMIZED` is specific to this project and does not exist in the Python reference converter. It may rewrite indexed palettes when a smaller LVGL indexed tier can be used without changing the visible result.
- Fully transparent RGB variants may be normalized or merged during indexed conversion so visually identical transparent pixels do not consume multiple palette entries.
- `ARGB8888_PREMULTIPLIED` is decoded back to straight-alpha RGBA/PNG by un-premultiplying the stored RGB channels, instead of reproducing the Python script's decode behavior byte-for-byte.
- Premultiplication math is not identical in every format. In particular, some Rust paths use `/ 255` where the Python script uses `>> 8`, so `premultiply=true` output is not guaranteed to be byte-identical to the reference implementation.

## License

MIT
