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

// Convert to LVGL binary format
const bin = imageToBin(png, 'ARGB8888', 0x000000, 1, false, 'NONE')
writeFileSync('icon.bin', bin)

// Convert to LVGL C source file
const cSource = imageToC(png, 'icon.png', null, 'RGB565', 0x000000, 1, false, 'LZ4')
writeFileSync('icon.c', cSource)

// Convert LVGL binary back to PNG
const pngOut = lvglToPng(bin, false)
writeFileSync('icon_out.png', pngOut)
```

### Browser

The browser entry point loads the WASM build automatically:

```js
import { imageToBin } from 'img2lv'

const file = await fetch('/icon.png').then((r) => r.arrayBuffer())
const bin = imageToBin(Buffer.from(file), 'ARGB8888', 0, 1, false, 'NONE')
```

> **Note:** C array parsing (`isCArray = true`) is not available in the browser build.

## API

### `imageToBin(input, cf, background, align, premultiply, compress)`

Convert an encoded image to LVGL binary format.

| Parameter     | Type      | Description                                                                |
| ------------- | --------- | -------------------------------------------------------------------------- |
| `input`       | `Buffer`  | Encoded image bytes (PNG, JPEG, WebP, BMP, etc.)                           |
| `cf`          | `string`  | Color format (see table below), or `"AUTO"`                                |
| `background`  | `number`  | Background color as 24-bit RGB (e.g. `0x000000`) for formats without alpha |
| `align`       | `number`  | Stride alignment in bytes (minimum `1`)                                    |
| `premultiply` | `boolean` | Whether to premultiply RGB channels with alpha                             |
| `compress`    | `string`  | `"NONE"`, `"RLE"`, or `"LZ4"`                                              |

**Returns:** `Buffer` — LVGL binary data (12-byte header + pixel data)

### `imageToC(input, inputName, outputName, cf, background, align, premultiply, compress)`

Convert an encoded image to a LVGL C source file.

| Parameter     | Type             | Description                                               |
| ------------- | ---------------- | --------------------------------------------------------- |
| `input`       | `Buffer`         | Encoded image bytes                                       |
| `inputName`   | `string`         | Original filename (used for variable naming)              |
| `outputName`  | `string \| null` | Override variable name, or `null` to derive from filename |
| `cf`          | `string`         | Color format (see table below), or `"AUTO"`               |
| `background`  | `number`         | Background color                                          |
| `align`       | `number`         | Stride alignment                                          |
| `premultiply` | `boolean`        | Premultiply alpha                                         |
| `compress`    | `string`         | `"NONE"`, `"RLE"`, or `"LZ4"`                             |

**Returns:** `string` — C source code containing the `lv_image_dsc_t` descriptor

### `lvglToPng(input, isCArray)`

Convert LVGL image data back to PNG.

| Parameter  | Type      | Description                                    |
| ---------- | --------- | ---------------------------------------------- |
| `input`    | `Buffer`  | LVGL binary data, or UTF-8 encoded C source    |
| `isCArray` | `boolean` | `true` if input is C source, `false` if binary |

**Returns:** `Buffer` — PNG file bytes

### `lvglToRgba(input, isCArray)`

Decode LVGL image data to raw RGBA pixels.

| Parameter  | Type      | Description                                    |
| ---------- | --------- | ---------------------------------------------- |
| `input`    | `Buffer`  | LVGL binary data, or UTF-8 encoded C source    |
| `isCArray` | `boolean` | `true` if input is C source, `false` if binary |

**Returns:** `Buffer` — Raw RGBA pixel data (4 bytes per pixel, row-major)

### `lvglWidth(input, isCArray)` / `lvglHeight(input, isCArray)`

Get image dimensions from LVGL data.

**Returns:** `number`

## Supported Color Formats

| Format                    | Description                                              |
| ------------------------- | -------------------------------------------------------- |
| `I1` / `I2` / `I4` / `I8` | Indexed with 2 / 4 / 16 / 256 color palette              |
| `A1` / `A2` / `A4` / `A8` | Alpha-only                                               |
| `AL88`                    | 8-bit luminance + 8-bit alpha                            |
| `L8`                      | 8-bit grayscale                                          |
| `RGB565`                  | 16-bit RGB (little-endian)                               |
| `RGB565_SWAPPED`          | 16-bit RGB (big-endian)                                  |
| `RGB565A8`                | 16-bit RGB + separate alpha plane                        |
| `ARGB8565`                | 16-bit RGB + inline alpha byte                           |
| `RGB888`                  | 24-bit RGB                                               |
| `ARGB8888`                | 32-bit RGBA                                              |
| `XRGB8888`                | 32-bit RGB (alpha forced to 0xFF)                        |
| `ARGB8888_PREMULTIPLIED`  | 32-bit premultiplied RGBA                                |
| `AUTO`                    | Automatically select indexed format based on color count |

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
yarn lint
```

### Cross-compilation

To build for a specific target:

```bash
yarn build --target aarch64-apple-darwin
yarn build --target x86_64-unknown-linux-musl -x
yarn build --target wasm32-wasip1-threads
```

For musl targets, [Zig](https://ziglang.org/) and `cargo-zigbuild` are required.

## Acknowledgements

This project's conversion logic is based on the official LVGL Python conversion script, which can be found in the `reference/` directory.

## License

MIT
