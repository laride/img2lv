#![deny(clippy::all)]

mod convert;

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Color format accepted by the conversion functions.
/// Includes the two special request modes AUTO and OPTIMIZED in addition to all
/// concrete LVGL color formats.
#[napi(string_enum)]
#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
pub enum ColorFormat {
  /// Automatically pick the smallest indexed tier (I1/I2/I4/I8) based on the
  /// number of distinct colors in the source image.
  AUTO,
  /// Like AUTO but also merges redundant transparent palette entries so the
  /// palette may shrink to a smaller tier.
  OPTIMIZED,
  UNKNOWN,
  RAW,
  RAW_ALPHA,
  L8,
  I1,
  I2,
  I4,
  I8,
  A1,
  A2,
  A4,
  A8,
  AL88,
  ARGB8888,
  XRGB8888,
  RGB565,
  ARGB8565,
  RGB565A8,
  RGB888,
  ARGB8888_PREMULTIPLIED,
  RGB565_SWAPPED,
}

/// Compression method for the binary payload.
#[napi(string_enum)]
#[allow(clippy::upper_case_acronyms)]
pub enum CompressMethod {
  NONE,
  RLE,
  LZ4,
}

// ---------------------------------------------------------------------------
// Internal conversions: napi enum → convert types
// ---------------------------------------------------------------------------

fn cf_to_color_request(cf: ColorFormat) -> convert::ColorRequest {
  use convert::ColorFormat as CF;
  match cf {
    ColorFormat::AUTO => convert::ColorRequest::Auto,
    ColorFormat::OPTIMIZED => convert::ColorRequest::Optimized,
    ColorFormat::UNKNOWN => convert::ColorRequest::Explicit(CF::Unknown),
    ColorFormat::RAW => convert::ColorRequest::Explicit(CF::Raw),
    ColorFormat::RAW_ALPHA => convert::ColorRequest::Explicit(CF::RawAlpha),
    ColorFormat::L8 => convert::ColorRequest::Explicit(CF::L8),
    ColorFormat::I1 => convert::ColorRequest::Explicit(CF::I1),
    ColorFormat::I2 => convert::ColorRequest::Explicit(CF::I2),
    ColorFormat::I4 => convert::ColorRequest::Explicit(CF::I4),
    ColorFormat::I8 => convert::ColorRequest::Explicit(CF::I8),
    ColorFormat::A1 => convert::ColorRequest::Explicit(CF::A1),
    ColorFormat::A2 => convert::ColorRequest::Explicit(CF::A2),
    ColorFormat::A4 => convert::ColorRequest::Explicit(CF::A4),
    ColorFormat::A8 => convert::ColorRequest::Explicit(CF::A8),
    ColorFormat::AL88 => convert::ColorRequest::Explicit(CF::Al88),
    ColorFormat::ARGB8888 => convert::ColorRequest::Explicit(CF::Argb8888),
    ColorFormat::XRGB8888 => convert::ColorRequest::Explicit(CF::Xrgb8888),
    ColorFormat::RGB565 => convert::ColorRequest::Explicit(CF::Rgb565),
    ColorFormat::ARGB8565 => convert::ColorRequest::Explicit(CF::Argb8565),
    ColorFormat::RGB565A8 => convert::ColorRequest::Explicit(CF::Rgb565A8),
    ColorFormat::RGB888 => convert::ColorRequest::Explicit(CF::Rgb888),
    ColorFormat::ARGB8888_PREMULTIPLIED => {
      convert::ColorRequest::Explicit(CF::Argb8888Premultiplied)
    }
    ColorFormat::RGB565_SWAPPED => convert::ColorRequest::Explicit(CF::Rgb565Swapped),
  }
}

fn compress_to_method(m: CompressMethod) -> convert::CompressMethod {
  match m {
    CompressMethod::NONE => convert::CompressMethod::None,
    CompressMethod::RLE => convert::CompressMethod::Rle,
    CompressMethod::LZ4 => convert::CompressMethod::Lz4,
  }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Options for converting an image to LVGL format.
/// All fields except `cf` are optional and fall back to sensible defaults.
#[napi(object)]
pub struct ConvertOptions {
  /// Color format. Accepts any `ColorFormat` string value (e.g. `'RGB565'`).
  #[napi(ts_type = "`${ColorFormat}`")]
  pub cf: ColorFormat,
  /// Background color packed as 0xRRGGBB. Defaults to `0`.
  pub background: Option<u32>,
  /// Stride alignment in bytes (any positive integer). Defaults to `1`.
  pub align: Option<u32>,
  /// Pre-multiply alpha into RGB channels. Defaults to `false`.
  pub premultiply: Option<bool>,
  /// Compression method. Accepts any `CompressMethod` string value. Defaults to `'NONE'`.
  #[napi(ts_type = "`${CompressMethod}`")]
  pub compress: Option<CompressMethod>,
  /// Enable dithering for RGB565 output. Defaults to `false`.
  pub rgb565_dither: Option<bool>,
  /// Enable Nema GFX compatible output. Defaults to `false`.
  pub nema_gfx: Option<bool>,
}

fn options_to_convert(o: ConvertOptions) -> convert::ConvertOptions {
  convert::ConvertOptions {
    cf: cf_to_color_request(o.cf),
    background: o.background.unwrap_or(0),
    align: o.align.unwrap_or(1) as usize,
    premultiply: o.premultiply.unwrap_or(false),
    compress: compress_to_method(o.compress.unwrap_or(CompressMethod::NONE)),
    rgb565_dither: o.rgb565_dither.unwrap_or(false),
    nema_gfx: o.nema_gfx.unwrap_or(false),
  }
}

#[napi]
pub fn image_to_bin(input: Buffer, options: ConvertOptions) -> Result<Buffer> {
  let result =
    convert::image_to_lvgl_bin(&input, options_to_convert(options)).map_err(to_napi_err)?;
  Ok(result.into())
}

#[napi]
pub fn image_to_c(
  input: Buffer,
  input_name: String,
  output_name: Option<String>,
  options: ConvertOptions,
) -> Result<String> {
  convert::image_to_lvgl_c(
    &input,
    &input_name,
    output_name.as_deref(),
    options_to_convert(options),
  )
  .map_err(to_napi_err)
}

/// Options for decoding an LVGL image back to a standard format.
#[napi(object)]
pub struct DecodeOptions {
  /// Set to `true` when the input is a C array produced by `imageToC`. Defaults to `false`.
  pub is_c_array: Option<bool>,
}

fn decode_is_c_array(opts: Option<DecodeOptions>) -> bool {
  opts.and_then(|o| o.is_c_array).unwrap_or(false)
}

#[napi]
pub fn lvgl_to_png(input: Buffer, options: Option<DecodeOptions>) -> Result<Buffer> {
  let result = convert::lvgl_to_png(&input, decode_is_c_array(options)).map_err(to_napi_err)?;
  Ok(result.into())
}

#[napi]
pub fn lvgl_to_rgba(input: Buffer, options: Option<DecodeOptions>) -> Result<Buffer> {
  let result = convert::lvgl_to_rgba(&input, decode_is_c_array(options)).map_err(to_napi_err)?;
  Ok(result.into())
}

#[napi]
pub fn lvgl_width(input: Buffer, options: Option<DecodeOptions>) -> Result<u32> {
  convert::lvgl_width(&input, decode_is_c_array(options))
    .map(|v| v as u32)
    .map_err(to_napi_err)
}

#[napi]
pub fn lvgl_height(input: Buffer, options: Option<DecodeOptions>) -> Result<u32> {
  convert::lvgl_height(&input, decode_is_c_array(options))
    .map(|v| v as u32)
    .map_err(to_napi_err)
}

fn to_napi_err(err: convert::LvglError) -> Error {
  Error::from_reason(err.to_string())
}
