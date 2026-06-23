use base64::Engine;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba, RgbaImage};
use lz4_flex::block::{compress as lz4_compress, decompress as lz4_decompress};
use png::{
  BitDepth as PngBitDepth, ColorType as PngColorType, Decoder as PngDecoder, Transformations,
};
use std::fmt::Write as _;
use std::io::{BufReader, Cursor};
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use regex::Regex;

pub type Result<T> = std::result::Result<T, LvglError>;

#[derive(Debug, Error)]
pub enum LvglError {
  #[error("invalid color format: {0}")]
  InvalidColorFormat(String),
  #[error("invalid compression method: {0}")]
  InvalidCompression(String),
  #[error("format error: {0}")]
  Format(String),
  #[error("parameter error: {0}")]
  Parameter(String),
  #[error("image error: {0}")]
  Image(#[from] image::ImageError),
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
  #[cfg(not(target_arch = "wasm32"))]
  #[error("regex error: {0}")]
  Regex(#[from] regex::Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ColorFormat {
  Unknown = 0x00,
  Raw = 0x01,
  RawAlpha = 0x02,
  L8 = 0x06,
  I1 = 0x07,
  I2 = 0x08,
  I4 = 0x09,
  I8 = 0x0A,
  A1 = 0x0B,
  A2 = 0x0C,
  A4 = 0x0D,
  A8 = 0x0E,
  Rgb888 = 0x0F,
  Argb8888 = 0x10,
  Xrgb8888 = 0x11,
  Rgb565 = 0x12,
  Argb8565 = 0x13,
  Rgb565A8 = 0x14,
  Al88 = 0x15,
  Argb8888Premultiplied = 0x1A,
  Rgb565Swapped = 0x1B,
}

impl ColorFormat {
  pub fn parse(value: &str) -> Result<Self> {
    let normalized = value.trim().to_ascii_uppercase();
    let cf = match normalized.as_str() {
      "UNKNOWN" => Self::Unknown,
      "RAW" => Self::Raw,
      "RAW_ALPHA" => Self::RawAlpha,
      "L8" => Self::L8,
      "I1" => Self::I1,
      "I2" => Self::I2,
      "I4" => Self::I4,
      "I8" => Self::I8,
      "A1" => Self::A1,
      "A2" => Self::A2,
      "A4" => Self::A4,
      "A8" => Self::A8,
      "AL88" => Self::Al88,
      "ARGB8888" => Self::Argb8888,
      "XRGB8888" => Self::Xrgb8888,
      "RGB565" => Self::Rgb565,
      "RGB565_SWAPPED" => Self::Rgb565Swapped,
      "RGB565A8" => Self::Rgb565A8,
      "ARGB8565" => Self::Argb8565,
      "RGB888" => Self::Rgb888,
      "ARGB8888_PREMULTIPLIED" => Self::Argb8888Premultiplied,
      _ => return Err(LvglError::InvalidColorFormat(value.to_string())),
    };
    Ok(cf)
  }

  pub fn from_byte(value: u8) -> Result<Self> {
    let cf = match value & 0x1f {
      0x00 => Self::Unknown,
      0x01 => Self::Raw,
      0x02 => Self::RawAlpha,
      0x06 => Self::L8,
      0x07 => Self::I1,
      0x08 => Self::I2,
      0x09 => Self::I4,
      0x0A => Self::I8,
      0x0B => Self::A1,
      0x0C => Self::A2,
      0x0D => Self::A4,
      0x0E => Self::A8,
      0x0F => Self::Rgb888,
      0x10 => Self::Argb8888,
      0x11 => Self::Xrgb8888,
      0x12 => Self::Rgb565,
      0x13 => Self::Argb8565,
      0x14 => Self::Rgb565A8,
      0x15 => Self::Al88,
      0x1A => Self::Argb8888Premultiplied,
      0x1B => Self::Rgb565Swapped,
      _ => return Err(LvglError::InvalidColorFormat(format!("0x{value:02x}"))),
    };
    Ok(cf)
  }

  pub fn name(self) -> &'static str {
    match self {
      Self::Unknown => "UNKNOWN",
      Self::Raw => "RAW",
      Self::RawAlpha => "RAW_ALPHA",
      Self::L8 => "L8",
      Self::I1 => "I1",
      Self::I2 => "I2",
      Self::I4 => "I4",
      Self::I8 => "I8",
      Self::A1 => "A1",
      Self::A2 => "A2",
      Self::A4 => "A4",
      Self::A8 => "A8",
      Self::Rgb888 => "RGB888",
      Self::Argb8888 => "ARGB8888",
      Self::Xrgb8888 => "XRGB8888",
      Self::Rgb565 => "RGB565",
      Self::Argb8565 => "ARGB8565",
      Self::Rgb565A8 => "RGB565A8",
      Self::Al88 => "AL88",
      Self::Argb8888Premultiplied => "ARGB8888_PREMULTIPLIED",
      Self::Rgb565Swapped => "RGB565_SWAPPED",
    }
  }

  pub fn bpp(self) -> usize {
    match self {
      Self::I1 | Self::A1 => 1,
      Self::I2 | Self::A2 => 2,
      Self::I4 | Self::A4 => 4,
      Self::L8 | Self::I8 | Self::A8 => 8,
      Self::Al88 | Self::Rgb565 | Self::Rgb565Swapped | Self::Rgb565A8 => 16,
      Self::Rgb888 | Self::Argb8565 => 24,
      Self::Argb8888 | Self::Xrgb8888 | Self::Argb8888Premultiplied => 32,
      _ => 0,
    }
  }

  pub fn ncolors(self) -> usize {
    match self {
      Self::I1 => 2,
      Self::I2 => 4,
      Self::I4 => 16,
      Self::I8 => 256,
      _ => 0,
    }
  }

  pub fn has_alpha(self) -> bool {
    matches!(
      self,
      Self::I1
        | Self::I2
        | Self::I4
        | Self::I8
        | Self::A1
        | Self::A2
        | Self::A4
        | Self::A8
        | Self::Al88
        | Self::Argb8888
        | Self::Xrgb8888
        | Self::Argb8565
        | Self::Rgb565A8
        | Self::Argb8888Premultiplied
    )
  }

  pub fn supports_premultiply(self) -> bool {
    matches!(
      self,
      Self::I1 | Self::I2 | Self::I4 | Self::I8 | Self::Argb8888 | Self::Argb8565 | Self::Rgb565A8
    )
  }

  pub fn is_indexed(self) -> bool {
    self.ncolors() != 0
  }

  pub fn is_alpha_only(self) -> bool {
    matches!(self, Self::A1 | Self::A2 | Self::A4 | Self::A8)
  }

  pub fn is_colormap(self) -> bool {
    matches!(
      self,
      Self::Argb8888
        | Self::Rgb888
        | Self::Xrgb8888
        | Self::Rgb565A8
        | Self::Argb8565
        | Self::Rgb565
        | Self::Rgb565Swapped
        | Self::Argb8888Premultiplied
    )
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorRequest {
  Auto,
  Optimized,
  Explicit(ColorFormat),
}

impl ColorRequest {
  pub fn parse(value: &str) -> Result<Self> {
    let normalized = value.trim().to_ascii_uppercase();
    match normalized.as_str() {
      "AUTO" => Ok(Self::Auto),
      "OPTIMIZED" => Ok(Self::Optimized),
      _ => Ok(Self::Explicit(ColorFormat::parse(value)?)),
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CompressMethod {
  None = 0x00,
  Rle = 0x01,
  Lz4 = 0x02,
}

impl CompressMethod {
  pub fn parse(value: &str) -> Result<Self> {
    match value.trim().to_ascii_uppercase().as_str() {
      "NONE" => Ok(Self::None),
      "RLE" => Ok(Self::Rle),
      "LZ4" => Ok(Self::Lz4),
      _ => Err(LvglError::InvalidCompression(value.to_string())),
    }
  }

  pub fn from_u32(value: u32) -> Result<Self> {
    match value {
      0 => Ok(Self::None),
      1 => Ok(Self::Rle),
      2 => Ok(Self::Lz4),
      _ => Err(LvglError::InvalidCompression(value.to_string())),
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct ConvertOptions {
  pub cf: ColorRequest,
  pub background: u32,
  pub align: usize,
  pub premultiply: bool,
  pub compress: CompressMethod,
  pub rgb565_dither: bool,
  pub nema_gfx: bool,
}

impl Default for ConvertOptions {
  fn default() -> Self {
    Self {
      cf: ColorRequest::Explicit(ColorFormat::I8),
      background: 0,
      align: 1,
      premultiply: false,
      compress: CompressMethod::None,
      rgb565_dither: false,
      nema_gfx: false,
    }
  }
}

#[derive(Clone, Debug)]
pub struct LvglImage {
  pub cf: ColorFormat,
  pub w: u16,
  pub h: u16,
  pub stride: u16,
  pub flags: u16,
  pub data: Vec<u8>,
  pub premultiplied: bool,
}

impl LvglImage {
  pub fn from_dynamic(img: DynamicImage, options: ConvertOptions) -> Result<Self> {
    let rgba = img.to_rgba8();
    Self::from_rgba(&rgba, options)
  }

  pub fn from_encoded(data: &[u8], options: ConvertOptions) -> Result<Self> {
    if let Some(image) = try_from_indexed_png(data, &options)? {
      return finalize_image(image, options);
    }
    Self::from_dynamic(image::load_from_memory(data)?, options)
  }

  pub fn from_rgba(rgba: &RgbaImage, options: ConvertOptions) -> Result<Self> {
    let (w, h) = rgba.dimensions();
    validate_image_dimensions_u32(w, h)?;
    if options.align == 0 {
      return Err(LvglError::Parameter("align must be at least 1".to_string()));
    }
    let normalize_transparent = !matches!(options.cf, ColorRequest::Auto);
    let cf = match options.cf {
      ColorRequest::Auto => auto_indexed_cf(rgba, false),
      ColorRequest::Optimized => auto_indexed_cf(rgba, true),
      ColorRequest::Explicit(cf) => cf,
    };
    validate_premultiply_option(cf, options.premultiply)?;
    let image = if cf.is_indexed() {
      rgba_to_indexed(rgba, cf, options.nema_gfx, normalize_transparent)?
    } else if cf.is_alpha_only() {
      rgba_to_alpha_only(rgba, cf)?
    } else if cf == ColorFormat::Al88 {
      rgba_to_al88(rgba)?
    } else if cf == ColorFormat::L8 {
      rgba_to_l8(rgba, options.background)?
    } else if cf.is_colormap() {
      rgba_to_colormap(rgba, cf, options.background, options.rgb565_dither)?
    } else {
      return Err(LvglError::InvalidColorFormat(cf.name().to_string()));
    };
    finalize_image(image, options)
  }

  pub fn from_data(cf: ColorFormat, w: u16, h: u16, data: Vec<u8>, stride: u16) -> Result<Self> {
    validate_image_dimensions_u16(w, h)?;
    let stride = if stride == 0 {
      validate_stride(default_stride(cf, w as usize))?
    } else {
      let s = validate_stride(stride as usize)?;
      let min = default_stride(cf, w as usize);
      if (s as usize) < min {
        return Err(LvglError::Parameter(format!(
          "stride is too small: {s}, minimal: {min}"
        )));
      }
      s
    };
    let expected = data_len(cf, w as usize, h as usize, stride as usize);
    if data.len() != expected {
      return Err(LvglError::Parameter(format!(
        "data length error got: {}, expect: {}",
        data.len(),
        expected
      )));
    }
    Ok(Self {
      cf,
      w,
      h,
      stride,
      flags: 0,
      data,
      premultiplied: false,
    })
  }

  pub fn from_bin(data: &[u8]) -> Result<Self> {
    let header = LvglHeader::parse(data)?;
    let payload = &data[12..];
    let raw = if header.flags & 0x08 != 0 {
      decompress_wrapped(payload, header.cf)?
    } else {
      payload.to_vec()
    };
    let mut image = Self::from_data(header.cf, header.w, header.h, raw, header.stride)?;
    image.flags = header.flags;
    image.premultiplied = header.flags & 0x01 != 0;
    Ok(image)
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn from_c_array(source: &str) -> Result<Self> {
    let cf_re = Regex::new(r"\.cf\s*=\s*LV_COLOR_FORMAT_([A-Z0-9_]+)")?;
    let num = |field: &str| -> Result<u16> {
      let re = Regex::new(&format!(r"\.{field}\s*=\s*(\d+)"))?;
      let caps = re
        .captures(source)
        .ok_or_else(|| LvglError::Format(format!("missing .{field}")))?;
      caps[1]
        .parse::<u16>()
        .map_err(|_| LvglError::Format(format!("invalid .{field}")))
    };
    let cf_caps = cf_re
      .captures(source)
      .ok_or_else(|| LvglError::Format("missing .cf".to_string()))?;
    let cf = match ColorRequest::parse(&cf_caps[1])? {
      ColorRequest::Explicit(cf) => cf,
      ColorRequest::Auto | ColorRequest::Optimized => {
        return Err(LvglError::Format(
          "AUTO/OPTIMIZED are not valid in C array".to_string(),
        ))
      }
    };
    let w = num("w")?;
    let h = num("h")?;
    let stride = num("stride")?;
    let data = parse_c_bytes(source)?;
    let compressed = source.contains("LV_IMAGE_FLAGS_COMPRESSED");
    let premultiplied = source.contains("LV_IMAGE_FLAGS_PREMULTIPLIED");
    let raw = if compressed {
      decompress_wrapped(&data, cf)?
    } else {
      data
    };
    let mut image = Self::from_data(cf, w, h, raw, stride)?;
    image.flags = if compressed { 0x08 } else { 0 } | if premultiplied { 0x01 } else { 0 };
    image.premultiplied = premultiplied;
    Ok(image)
  }

  pub fn adjust_stride_align(&mut self, align: usize) -> Result<()> {
    if align == 0 {
      return Err(LvglError::Parameter("align must be at least 1".to_string()));
    }
    let new_stride = align_to(default_stride(self.cf, self.w as usize), align)?;
    self.adjust_stride(new_stride)
  }

  pub fn adjust_stride(&mut self, new_stride: usize) -> Result<()> {
    let new_stride = validate_stride(new_stride)? as usize;
    let old_stride = self.stride as usize;
    if new_stride == old_stride {
      return Ok(());
    }
    let min_stride = default_stride(self.cf, self.w as usize);
    if new_stride < min_stride {
      return Err(LvglError::Parameter(format!(
        "stride is too small: {new_stride}, minimal: {min_stride}"
      )));
    }
    let palette_size = self.cf.ncolors() * 4;
    let h = self.h as usize;
    let mut out = Vec::with_capacity(data_len(self.cf, self.w as usize, h, new_stride));
    out.extend_from_slice(&self.data[..palette_size]);
    let color_len = old_stride * h;
    change_stride(
      &self.data[palette_size..palette_size + color_len],
      h,
      old_stride,
      new_stride,
      &mut out,
    );
    if self.cf == ColorFormat::Rgb565A8 {
      change_stride(
        &self.data[palette_size + color_len..],
        h,
        old_stride / 2,
        new_stride / 2,
        &mut out,
      );
    }
    self.stride = new_stride as u16;
    self.data = out;
    Ok(())
  }

  pub fn premultiply(&mut self) -> Result<()> {
    if self.premultiplied {
      return Err(LvglError::Parameter(
        "image already pre-multiplied".to_string(),
      ));
    }
    if !self.cf.has_alpha() {
      return Err(LvglError::Parameter(format!(
        "image has no alpha channel: {}",
        self.cf.name()
      )));
    }
    match self.cf {
      ColorFormat::Argb8888 => {
        // NOTE: The Python reference uses `>> 8` (divides by 256) which gives a max premultiplied
        // value of 254 for fully opaque pixels (255 * 255 >> 8 = 254). We use `/ 255` here for
        // mathematical correctness and consistency with the other formats in this function.
        let line_width = self.w as usize * 4;
        for y in 0..self.h as usize {
          let row = y * self.stride as usize;
          for i in (0..line_width).step_by(4) {
            let a = self.data[row + i + 3] as u32;
            self.data[row + i] = (self.data[row + i] as u32 * a / 255) as u8;
            self.data[row + i + 1] = (self.data[row + i + 1] as u32 * a / 255) as u8;
            self.data[row + i + 2] = (self.data[row + i + 2] as u32 * a / 255) as u8;
          }
        }
      }
      ColorFormat::Rgb565A8 => {
        let line_width = self.w as usize * 2;
        let color_plane = self.stride as usize * self.h as usize;
        for y in 0..self.h as usize {
          let rgb_row = y * self.stride as usize;
          let alpha_row = color_plane + y * (self.stride as usize / 2);
          for i in (0..line_width).step_by(2) {
            let a = self.data[alpha_row + i / 2] as u32;
            let p = u16::from_le_bytes([self.data[rgb_row + i], self.data[rgb_row + i + 1]]);
            let r = (((p >> 11) & 0x1f) as u32 * a / 255) as u16;
            let g = (((p >> 5) & 0x3f) as u32 * a / 255) as u16;
            let b = ((p & 0x1f) as u32 * a / 255) as u16;
            self.data[rgb_row + i..rgb_row + i + 2]
              .copy_from_slice(&((r << 11) | (g << 5) | b).to_le_bytes());
          }
        }
      }
      ColorFormat::Argb8565 => {
        let line_width = self.w as usize * 3;
        for y in 0..self.h as usize {
          let row = y * self.stride as usize;
          for i in (0..line_width).step_by(3) {
            let a = self.data[row + i + 2] as u32;
            let p = (self.data[row + i + 1] as u16) << 8 | self.data[row + i] as u16;
            let r = (((p >> 11) & 0x1f) as u32 * a / 255) as u16;
            let g = (((p >> 5) & 0x3f) as u32 * a / 255) as u16;
            let b = ((p & 0x1f) as u32 * a / 255) as u16;
            let color = (r << 11) | (g << 5) | b;
            self.data[row + i..row + i + 2].copy_from_slice(&color.to_le_bytes());
          }
        }
      }
      cf if cf.is_indexed() => {
        // NOTE: The Python reference uses `>> 8` here; we use `/ 255` for correctness.
        for i in (0..cf.ncolors() * 4).step_by(4) {
          let a = self.data[i + 3] as u32;
          self.data[i] = (self.data[i] as u32 * a / 255) as u8;
          self.data[i + 1] = (self.data[i + 1] as u32 * a / 255) as u8;
          self.data[i + 2] = (self.data[i + 2] as u32 * a / 255) as u8;
        }
      }
      _ => {
        return Err(LvglError::Parameter(format!(
          "premultiply not supported for {}",
          self.cf.name()
        )));
      }
    }
    self.premultiplied = true;
    Ok(())
  }

  pub fn to_bin(&self, compress: CompressMethod) -> Result<Vec<u8>> {
    let mut flags = 0u16;
    if compress != CompressMethod::None {
      flags |= 0x08;
    }
    if self.premultiplied {
      flags |= 0x01;
    }
    let mut out = LvglHeader {
      cf: self.cf,
      flags,
      w: self.w,
      h: self.h,
      stride: self.stride,
    }
    .to_bytes();
    out.extend_from_slice(&compress_wrapped(self.cf, compress, &self.data)?);
    Ok(out)
  }

  pub fn to_c_array(
    &self,
    filename: &str,
    output_name: Option<&str>,
    compress: CompressMethod,
  ) -> Result<String> {
    let stem = output_name
      .map(str::to_string)
      .unwrap_or_else(|| c_var_name(filename));
    let data = compress_wrapped(self.cf, compress, &self.data)?;
    let mut flags = String::from("0");
    if compress != CompressMethod::None {
      flags.push_str(" | LV_IMAGE_FLAGS_COMPRESSED");
    }
    if self.premultiplied {
      flags.push_str(" | LV_IMAGE_FLAGS_PREMULTIPLIED");
    }
    let macro_name = format!("LV_ATTRIBUTE_{}", stem.to_ascii_uppercase());
    let version = env!("CARGO_PKG_VERSION");
    let mut out = format!(
      r#"
/**
 * Auto-generated by img2lv v{version}
 * Bugs and issues: https://github.com/laride/img2lv
 */
#if defined(LV_LVGL_H_INCLUDE_SIMPLE)
#include "lvgl.h"
#elif defined(LV_LVGL_H_INCLUDE_SYSTEM)
#include <lvgl.h>
#elif defined(LV_BUILD_TEST)
#include "../lvgl.h"
#else
#include "lvgl/lvgl.h"
#endif

#ifndef LV_ATTRIBUTE_MEM_ALIGN
#define LV_ATTRIBUTE_MEM_ALIGN
#endif

#ifndef {macro_name}
#define {macro_name}
#endif

static const
LV_ATTRIBUTE_MEM_ALIGN LV_ATTRIBUTE_LARGE_CONST {macro_name}
uint8_t {stem}_map[] = {{
"#
    );
    write_c_bytes(
      &mut out,
      &data,
      if compress == CompressMethod::None {
        self.stride as usize
      } else {
        16
      },
    );
    let _ = write!(
      out,
      r#"
}};

const lv_image_dsc_t {stem} = {{
  .header = {{
    .magic = LV_IMAGE_HEADER_MAGIC,
    .cf = LV_COLOR_FORMAT_{},
    .flags = {flags},
    .w = {},
    .h = {},
    .stride = {},
    .reserved_2 = 0,
  }},
  .data_size = sizeof({stem}_map),
  .data = {stem}_map,
  .reserved = NULL,
}};

"#,
      self.cf.name(),
      self.w,
      self.h,
      self.stride
    );
    Ok(out)
  }

  pub fn to_rgba_image(&self) -> Result<RgbaImage> {
    let mut clone = self.clone();
    clone.adjust_stride_align(1)?;
    unpack_to_rgba(&clone)
  }

  pub fn to_png_bytes(&self) -> Result<Vec<u8>> {
    let rgba = self.to_rgba_image()?;
    let mut cursor = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(rgba).write_to(&mut cursor, ImageFormat::Png)?;
    Ok(cursor.into_inner())
  }
}

#[derive(Clone, Copy, Debug)]
struct LvglHeader {
  cf: ColorFormat,
  flags: u16,
  w: u16,
  h: u16,
  stride: u16,
}

impl LvglHeader {
  fn parse(data: &[u8]) -> Result<Self> {
    if data.len() < 12 {
      return Err(LvglError::Format("invalid header length".to_string()));
    }
    if data[0] != 0x19 {
      return Err(LvglError::Format(format!(
        "invalid magic: 0x{:02x}",
        data[0]
      )));
    }
    Ok(Self {
      cf: ColorFormat::from_byte(data[1])?,
      flags: u16::from_le_bytes([data[2], data[3]]),
      w: u16::from_le_bytes([data[4], data[5]]),
      h: u16::from_le_bytes([data[6], data[7]]),
      stride: u16::from_le_bytes([data[8], data[9]]),
    })
  }

  fn to_bytes(self) -> Vec<u8> {
    let mut out = Vec::with_capacity(12);
    out.push(0x19);
    out.push(self.cf as u8);
    out.extend_from_slice(&self.flags.to_le_bytes());
    out.extend_from_slice(&self.w.to_le_bytes());
    out.extend_from_slice(&self.h.to_le_bytes());
    out.extend_from_slice(&self.stride.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
  }
}

pub fn image_to_lvgl_bin(input: &[u8], options: ConvertOptions) -> Result<Vec<u8>> {
  let image = LvglImage::from_encoded(input, options)?;
  image.to_bin(options.compress)
}

pub fn image_to_lvgl_c(
  input: &[u8],
  input_name: &str,
  output_name: Option<&str>,
  options: ConvertOptions,
) -> Result<String> {
  let image = LvglImage::from_encoded(input, options)?;
  image.to_c_array(input_name, output_name, options.compress)
}

pub fn lvgl_to_png(input: &[u8], is_c_array: bool) -> Result<Vec<u8>> {
  let image = if is_c_array {
    #[cfg(not(target_arch = "wasm32"))]
    {
      let source = std::str::from_utf8(input)
        .map_err(|_| LvglError::Format("C input is not valid UTF-8".to_string()))?;
      LvglImage::from_c_array(source)?
    }
    #[cfg(target_arch = "wasm32")]
    {
      let _ = input;
      return Err(LvglError::Format(
        "C array parsing is not supported on WASM target".to_string(),
      ));
    }
  } else {
    LvglImage::from_bin(input)?
  };
  image.to_png_bytes()
}

pub fn lvgl_to_rgba(input: &[u8], is_c_array: bool) -> Result<Vec<u8>> {
  let image = if is_c_array {
    #[cfg(not(target_arch = "wasm32"))]
    {
      let source = std::str::from_utf8(input)
        .map_err(|_| LvglError::Format("C input is not valid UTF-8".to_string()))?;
      LvglImage::from_c_array(source)?
    }
    #[cfg(target_arch = "wasm32")]
    {
      let _ = input;
      return Err(LvglError::Format(
        "C array parsing is not supported on WASM target".to_string(),
      ));
    }
  } else {
    LvglImage::from_bin(input)?
  };
  Ok(image.to_rgba_image()?.into_raw())
}

fn parse_lvgl(input: &[u8], is_c_array: bool) -> Result<LvglImage> {
  if is_c_array {
    #[cfg(not(target_arch = "wasm32"))]
    {
      let source = std::str::from_utf8(input)
        .map_err(|_| LvglError::Format("C input is not valid UTF-8".to_string()))?;
      LvglImage::from_c_array(source)
    }
    #[cfg(target_arch = "wasm32")]
    {
      let _ = input;
      Err(LvglError::Format(
        "C array parsing is not supported on WASM target".to_string(),
      ))
    }
  } else {
    LvglImage::from_bin(input)
  }
}

pub fn lvgl_width(input: &[u8], is_c_array: bool) -> Result<u16> {
  Ok(parse_lvgl(input, is_c_array)?.w)
}

pub fn lvgl_height(input: &[u8], is_c_array: bool) -> Result<u16> {
  Ok(parse_lvgl(input, is_c_array)?.h)
}

#[allow(dead_code)]
pub fn base64_png_from_lvgl(input: &[u8], is_c_array: bool) -> Result<String> {
  Ok(base64::engine::general_purpose::STANDARD.encode(lvgl_to_png(input, is_c_array)?))
}

// ---------------------------------------------------------------------------
// Indexed color quantization using exoquant (pure Rust, WASM-compatible)
// ---------------------------------------------------------------------------

fn rgba_to_indexed(
  rgba: &RgbaImage,
  cf: ColorFormat,
  nema_gfx: bool,
  normalize_transparent: bool,
) -> Result<LvglImage> {
  let capacity = cf.ncolors();
  let w = rgba.width() as usize;

  let pixels: Vec<exoquant::Color> = rgba
    .pixels()
    .map(|p| {
      let [r, g, b, a] = if normalize_transparent {
        normalize_transparent_pixel(p.0)
      } else {
        p.0
      };
      exoquant::Color::new(r, g, b, a)
    })
    .collect();

  let optimizer = exoquant::optimizer::KMeans;
  let ditherer = exoquant::ditherer::FloydSteinberg::new();
  let (pal, indexes) = exoquant::convert_to_indexed(&pixels, w, capacity, &optimizer, &ditherer);

  let mut palette: Vec<[u8; 4]> = pal.iter().map(|c| [c.r, c.g, c.b, c.a]).collect();
  while palette.len() < capacity {
    palette.push([255, 255, 255, 0]);
  }

  let mut data = Vec::with_capacity(capacity * 4 + packed_len(indexes.len(), cf.bpp()));
  for [r, g, b, a] in &palette {
    data.extend_from_slice(&[*b, *g, *r, *a]);
  }

  let indexes_u8: Vec<u8> = indexes.to_vec();

  if cf == ColorFormat::I8 {
    if nema_gfx {
      data.extend(indexes_u8.iter().map(|x| (x >> 4) | ((x & 0x0f) << 4)));
    } else {
      data.extend_from_slice(&indexes_u8);
    }
  } else {
    pack_indices(&indexes_u8, cf.bpp(), w, &mut data);
  }
  LvglImage::from_data(cf, rgba.width() as u16, rgba.height() as u16, data, 0)
}

fn try_from_indexed_png(data: &[u8], options: &ConvertOptions) -> Result<Option<LvglImage>> {
  let mode = match options.cf {
    ColorRequest::Auto => IndexedPngMode::Auto,
    ColorRequest::Optimized => IndexedPngMode::Optimized,
    ColorRequest::Explicit(cf) if cf.is_indexed() => IndexedPngMode::Explicit(cf),
    ColorRequest::Explicit(_) => return Ok(None),
  };
  let Some(metadata) = parse_indexed_png_metadata(data)? else {
    return Ok(None);
  };

  let cursor = Cursor::new(data);
  let mut decoder = PngDecoder::new(BufReader::new(cursor));
  decoder.set_transformations(Transformations::IDENTITY);
  let mut reader = decoder
    .read_info()
    .map_err(|e| LvglError::Format(format!("png decode error: {e}")))?;
  let palette_len = metadata.palette_rgb.len() / 3;
  if palette_len == 0 {
    return Ok(None);
  }

  let mut raw = vec![
    0;
    reader.output_buffer_size().ok_or_else(|| {
      LvglError::Format("png indexed output buffer size overflow".to_string())
    })?
  ];
  let frame = reader
    .next_frame(&mut raw)
    .map_err(|e| LvglError::Format(format!("png decode error: {e}")))?;

  validate_image_dimensions_u32(frame.width, frame.height)?;
  let indexes = unpack_png_indices(
    &raw[..frame.buffer_size()],
    metadata.bit_depth,
    frame.width as usize,
    frame.height as usize,
  )?;
  let w = frame.width as u16;
  let h = frame.height as u16;
  let original = build_indexed_image_from_palette(
    w,
    h,
    indexed_cf_for_palette_len(palette_len),
    &metadata.palette_rgb,
    metadata.trns.as_deref(),
    &indexes,
    options.nema_gfx,
  )?;

  let image = match mode {
    // Match the Python reference for AUTO: preserve indexed PNG palette/indexes as-is
    // and only pick the LVGL indexed tier from the source palette length.
    IndexedPngMode::Auto => original,
    IndexedPngMode::Explicit(cf) => {
      if palette_len > cf.ncolors() {
        return Ok(None);
      }
      build_indexed_image_from_palette(
        w,
        h,
        cf,
        &metadata.palette_rgb,
        metadata.trns.as_deref(),
        &indexes,
        options.nema_gfx,
      )?
    }
    IndexedPngMode::Optimized => {
      let optimized = optimize_indexed_png_palette(
        w,
        h,
        &metadata.palette_rgb,
        metadata.trns.as_deref(),
        &indexes,
        options.nema_gfx,
      )?;
      if optimized.data.len() < original.data.len() {
        optimized
      } else {
        original
      }
    }
  };

  Ok(Some(image))
}

fn rgba_to_alpha_only(rgba: &RgbaImage, cf: ColorFormat) -> Result<LvglImage> {
  let mut values = Vec::with_capacity((rgba.width() * rgba.height()) as usize);
  let shift = 8 - cf.bpp();
  let mask = (1u8 << cf.bpp()) - 1;
  for pixel in rgba.pixels() {
    values.push(if cf == ColorFormat::A8 {
      pixel[3]
    } else {
      (pixel[3] >> shift) & mask
    });
  }
  let mut data = Vec::new();
  if cf == ColorFormat::A8 {
    data = values;
  } else {
    pack_indices(&values, cf.bpp(), rgba.width() as usize, &mut data);
  }
  LvglImage::from_data(cf, rgba.width() as u16, rgba.height() as u16, data, 0)
}

fn rgba_to_al88(rgba: &RgbaImage) -> Result<LvglImage> {
  let mut data = Vec::with_capacity((rgba.width() * rgba.height() * 2) as usize);
  for p in rgba.pixels() {
    data.push(luma_srgb(p[0], p[1], p[2]));
    data.push(p[3]);
  }
  LvglImage::from_data(
    ColorFormat::Al88,
    rgba.width() as u16,
    rgba.height() as u16,
    data,
    0,
  )
}

fn rgba_to_l8(rgba: &RgbaImage, background: u32) -> Result<LvglImage> {
  let mut data = Vec::with_capacity((rgba.width() * rgba.height()) as usize);
  for p in rgba.pixels() {
    let (r, g, b, _) = color_pre_multiply(p[0], p[1], p[2], p[3], background);
    data.push(luma_srgb(r, g, b));
  }
  LvglImage::from_data(
    ColorFormat::L8,
    rgba.width() as u16,
    rgba.height() as u16,
    data,
    0,
  )
}

fn rgba_to_colormap(
  rgba: &RgbaImage,
  cf: ColorFormat,
  background: u32,
  dither: bool,
) -> Result<LvglImage> {
  let mut data = Vec::new();
  let mut alpha = Vec::new();
  for (x, y, p) in rgba.enumerate_pixels() {
    let mut r = p[0];
    let mut g = p[1];
    let mut b = p[2];
    let a = p[3];
    if dither
      && matches!(
        cf,
        ColorFormat::Rgb565
          | ColorFormat::Rgb565Swapped
          | ColorFormat::Rgb565A8
          | ColorFormat::Argb8565
      )
    {
      let id = (((y as usize) & 7) << 3) + ((x as usize) & 7);
      r = r.saturating_add(RED_THRESH[id]) & 0xf8;
      g = g.saturating_add(GREEN_THRESH[id]) & 0xfc;
      b = b.saturating_add(BLUE_THRESH[id]) & 0xf8;
    }
    match cf {
      ColorFormat::Argb8888 => data.extend_from_slice(&[b, g, r, a]),
      ColorFormat::Argb8888Premultiplied => data.extend_from_slice(&[
        (b as u16 * a as u16 / 255) as u8,
        (g as u16 * a as u16 / 255) as u8,
        (r as u16 * a as u16 / 255) as u8,
        a,
      ]),
      ColorFormat::Xrgb8888 => {
        let (r, g, b, _) = color_pre_multiply(r, g, b, a, background);
        data.extend_from_slice(&[b, g, r, 0xff]);
      }
      ColorFormat::Rgb888 => {
        let (r, g, b, _) = color_pre_multiply(r, g, b, a, background);
        data.extend_from_slice(&[b, g, r]);
      }
      ColorFormat::Rgb565 | ColorFormat::Rgb565Swapped => {
        let (r, g, b, _) = color_pre_multiply(r, g, b, a, background);
        let color = rgb565(r, g, b);
        if cf == ColorFormat::Rgb565 {
          data.extend_from_slice(&color.to_le_bytes());
        } else {
          data.extend_from_slice(&color.to_be_bytes());
        }
      }
      ColorFormat::Rgb565A8 => {
        data.extend_from_slice(&rgb565(r, g, b).to_le_bytes());
        alpha.push(a);
      }
      ColorFormat::Argb8565 => {
        data.extend_from_slice(&rgb565(r, g, b).to_le_bytes());
        data.push(a);
      }
      _ => return Err(LvglError::InvalidColorFormat(cf.name().to_string())),
    }
  }
  if cf == ColorFormat::Rgb565A8 {
    data.extend_from_slice(&alpha);
  }
  LvglImage::from_data(cf, rgba.width() as u16, rgba.height() as u16, data, 0)
}

fn unpack_to_rgba(img: &LvglImage) -> Result<RgbaImage> {
  let w = img.w as usize;
  let h = img.h as usize;
  let stride = img.stride as usize;
  let mut out = Vec::with_capacity(w * h * 4);
  if img.cf.is_indexed() {
    let palette_size = img.cf.ncolors() * 4;
    let palette = &img.data[..palette_size];
    let indexes = unpack_packed(&img.data[palette_size..], img.cf.bpp(), w, h, stride, false);
    for idx in indexes {
      let p = idx as usize * 4;
      out.extend_from_slice(&[palette[p + 2], palette[p + 1], palette[p], palette[p + 3]]);
    }
  } else if img.cf.is_alpha_only() {
    let values = unpack_packed(&img.data, img.cf.bpp(), w, h, stride, true);
    for a in values {
      out.extend_from_slice(&[0, 0, 0, a]);
    }
  } else {
    match img.cf {
      ColorFormat::L8 => {
        for y in 0..h {
          let row = y * stride;
          for x in 0..w {
            let l = img.data[row + x];
            out.extend_from_slice(&[l, l, l, 255]);
          }
        }
      }
      ColorFormat::Al88 => {
        for y in 0..h {
          let row = y * stride;
          for x in 0..w {
            let l = img.data[row + x * 2];
            let a = img.data[row + x * 2 + 1];
            out.extend_from_slice(&[l, l, l, a]);
          }
        }
      }
      ColorFormat::Rgb565 | ColorFormat::Rgb565Swapped => {
        for y in 0..h {
          let row = y * stride;
          for x in 0..w {
            let i = row + x * 2;
            let color = if img.cf == ColorFormat::Rgb565 {
              u16::from_le_bytes([img.data[i], img.data[i + 1]])
            } else {
              u16::from_be_bytes([img.data[i], img.data[i + 1]])
            };
            let [r, g, b] = unpack_rgb565(color);
            out.extend_from_slice(&[r, g, b, 255]);
          }
        }
      }
      ColorFormat::Rgb888 => {
        for y in 0..h {
          let row = y * stride;
          for x in 0..w {
            let i = row + x * 3;
            out.extend_from_slice(&[img.data[i + 2], img.data[i + 1], img.data[i], 255]);
          }
        }
      }
      ColorFormat::Argb8888 | ColorFormat::Xrgb8888 | ColorFormat::Argb8888Premultiplied => {
        // Stored layout: [B, G, R, A].  Copy as straight RGBA for now; if the image carries
        // premultiplied data (flag or ARGB8888_PREMULTIPLIED format) it will be un-premultiplied
        // in the post-processing step below.
        //
        // NOTE: The Python reference applies another round of alpha multiplication when decoding
        // ARGB8888_PREMULTIPLIED (effectively double-multiplying the already-premultiplied channels),
        // which may not produce the intended straight-alpha result. We un-premultiply instead to
        // recover straight-alpha RGBA for standard PNG/RGBA output.
        for y in 0..h {
          let row = y * stride;
          for x in 0..w {
            let i = row + x * 4;
            out.extend_from_slice(&[
              img.data[i + 2],
              img.data[i + 1],
              img.data[i],
              img.data[i + 3],
            ]);
          }
        }
      }
      ColorFormat::Argb8565 => {
        for y in 0..h {
          let row = y * stride;
          for x in 0..w {
            let i = row + x * 3;
            let [r, g, b] = unpack_rgb565(u16::from_le_bytes([img.data[i], img.data[i + 1]]));
            out.extend_from_slice(&[r, g, b, img.data[i + 2]]);
          }
        }
      }
      ColorFormat::Rgb565A8 => {
        let alpha_plane = stride * h;
        for y in 0..h {
          let rgb_row = y * stride;
          let alpha_row = alpha_plane + y * (stride / 2);
          for x in 0..w {
            let i = rgb_row + x * 2;
            let [r, g, b] = unpack_rgb565(u16::from_le_bytes([img.data[i], img.data[i + 1]]));
            out.extend_from_slice(&[r, g, b, img.data[alpha_row + x]]);
          }
        }
      }
      _ => return Err(LvglError::InvalidColorFormat(img.cf.name().to_string())),
    }
  }

  // Un-premultiply to recover straight-alpha RGBA that PNG viewers and standard RGBA buffers
  // expect.  This applies when:
  //   - img.premultiplied is true  — any format whose RGB was explicitly premultiplied via the
  //     LV_IMAGE_FLAGS_PREMULTIPLIED flag (ARGB8888, indexed, ARGB8565, RGB565A8, …)
  //   - cf == Argb8888Premultiplied — this format always stores premultiplied RGB regardless of
  //     the flag
  //
  // The inverse formula is:  channel_straight = channel_premult * 255 / alpha  (rounded)
  // For alpha == 0 all channels are set to 0 (fully transparent, RGB is undefined).
  // For alpha == 255 the operation is a no-op.
  let needs_unpremultiply = img.premultiplied || img.cf == ColorFormat::Argb8888Premultiplied;
  if needs_unpremultiply {
    for pixel in out.chunks_exact_mut(4) {
      let a = pixel[3];
      if a == 0 {
        pixel[0] = 0;
        pixel[1] = 0;
        pixel[2] = 0;
      } else if a < 255 {
        let a32 = a as u32;
        pixel[0] = ((pixel[0] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
        pixel[1] = ((pixel[1] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
        pixel[2] = ((pixel[2] as u32 * 255 + a32 / 2) / a32).min(255) as u8;
      }
    }
  }

  ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(img.w as u32, img.h as u32, out)
    .ok_or_else(|| LvglError::Format("failed to build output image".to_string()))
}

fn auto_indexed_cf(rgba: &RgbaImage, normalize_transparent: bool) -> ColorFormat {
  let mut colors = Vec::<[u8; 4]>::new();
  for p in rgba.pixels() {
    let pixel = if normalize_transparent {
      normalize_transparent_pixel(p.0)
    } else {
      p.0
    };
    if !colors.contains(&pixel) {
      colors.push(pixel);
      if colors.len() > 16 {
        return ColorFormat::I8;
      }
    }
  }
  match colors.len() {
    0..=2 => ColorFormat::I1,
    3..=4 => ColorFormat::I2,
    5..=16 => ColorFormat::I4,
    _ => ColorFormat::I8,
  }
}

fn default_stride(cf: ColorFormat, w: usize) -> usize {
  (w * cf.bpp()).div_ceil(8)
}

fn data_len(cf: ColorFormat, w: usize, h: usize, stride: usize) -> usize {
  let mut len = if cf.is_indexed() && w * h > 0 {
    cf.ncolors() * 4
  } else {
    0
  };
  len += stride * h;
  if cf == ColorFormat::Rgb565A8 {
    len += (stride / 2) * h;
  }
  len
}

fn align_to(value: usize, align: usize) -> Result<usize> {
  let rem = value % align;
  if rem == 0 {
    return Ok(value);
  }
  value.checked_add(align - rem).ok_or_else(|| {
    LvglError::Parameter(format!(
      "stride alignment overflow: value {value}, align {align}"
    ))
  })
}

fn change_stride(data: &[u8], h: usize, old_stride: usize, new_stride: usize, out: &mut Vec<u8>) {
  for y in 0..h {
    let row = &data[y * old_stride..(y + 1) * old_stride];
    out.extend_from_slice(&row[..old_stride.min(new_stride)]);
    if new_stride > old_stride {
      out.resize(out.len() + new_stride - old_stride, 0);
    }
  }
}

fn packed_len(count: usize, bpp: usize) -> usize {
  (count * bpp).div_ceil(8)
}

fn pack_indices(values: &[u8], bpp: usize, w: usize, out: &mut Vec<u8>) {
  let mask = (1u16 << bpp) - 1;
  for row in values.chunks(w) {
    let mut byte = 0u8;
    let mut used = 0usize;
    for value in row {
      byte |= (((*value as u16) & mask) as u8) << (8 - bpp - used);
      used += bpp;
      if used == 8 {
        out.push(byte);
        byte = 0;
        used = 0;
      }
    }
    if used != 0 {
      out.push(byte);
    }
  }
}

fn unpack_packed(
  data: &[u8],
  bpp: usize,
  w: usize,
  h: usize,
  stride: usize,
  alpha_extend: bool,
) -> Vec<u8> {
  if bpp == 8 {
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
      out.extend_from_slice(&data[y * stride..y * stride + w]);
    }
    return out;
  }
  let mut out = Vec::with_capacity(w * h);
  let mask = (1u8 << bpp) - 1;
  for y in 0..h {
    let row = &data[y * stride..(y + 1) * stride];
    'byte_loop: for byte in row {
      for shift in (0..8).step_by(bpp).map(|s| 8 - bpp - s) {
        let value = (byte >> shift) & mask;
        out.push(if alpha_extend {
          bit_extend(value, bpp)
        } else {
          value
        });
        if out.len() % w == 0 {
          break 'byte_loop;
        }
      }
    }
  }
  out.truncate(w * h);
  out
}

fn unpack_png_indices(data: &[u8], bit_depth: PngBitDepth, w: usize, h: usize) -> Result<Vec<u8>> {
  let bpp = match bit_depth {
    PngBitDepth::One => 1,
    PngBitDepth::Two => 2,
    PngBitDepth::Four => 4,
    PngBitDepth::Eight => 8,
    _ => {
      return Err(LvglError::Format(format!(
        "unsupported indexed png bit depth: {}",
        bit_depth as u8
      )))
    }
  };
  let stride = (w * bpp).div_ceil(8);
  Ok(unpack_packed(data, bpp, w, h, stride, false))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IndexedPngMode {
  Auto,
  Optimized,
  Explicit(ColorFormat),
}

fn indexed_cf_for_palette_len(palette_len: usize) -> ColorFormat {
  match palette_len {
    0..=2 => ColorFormat::I1,
    3..=4 => ColorFormat::I2,
    5..=16 => ColorFormat::I4,
    _ => ColorFormat::I8,
  }
}

fn build_indexed_image_from_palette(
  w: u16,
  h: u16,
  cf: ColorFormat,
  palette_rgb: &[u8],
  trns: Option<&[u8]>,
  indexes: &[u8],
  nema_gfx: bool,
) -> Result<LvglImage> {
  let palette_len = palette_rgb.len() / 3;
  let mut palette = Vec::with_capacity(palette_len);
  for i in 0..palette_len {
    let base = i * 3;
    palette.push([
      palette_rgb[base],
      palette_rgb[base + 1],
      palette_rgb[base + 2],
      trns.and_then(|alpha| alpha.get(i)).copied().unwrap_or(0xff),
    ]);
  }
  build_indexed_image_from_entries(w, h, cf, &palette, indexes, nema_gfx)
}

fn build_indexed_image_from_entries(
  w: u16,
  h: u16,
  cf: ColorFormat,
  palette: &[[u8; 4]],
  indexes: &[u8],
  nema_gfx: bool,
) -> Result<LvglImage> {
  if palette.len() > cf.ncolors() {
    return Err(LvglError::Parameter(format!(
      "palette too large: {}, capacity: {}",
      palette.len(),
      cf.ncolors()
    )));
  }
  if let Some(&bad) = indexes.iter().find(|&&i| i as usize >= palette.len()) {
    return Err(LvglError::Format(format!(
      "png index out of palette range: {}",
      bad
    )));
  }

  let mut data = Vec::with_capacity(cf.ncolors() * 4 + packed_len(indexes.len(), cf.bpp()));
  for [r, g, b, a] in palette {
    data.extend_from_slice(&[*b, *g, *r, *a]);
  }
  while data.len() < cf.ncolors() * 4 {
    data.extend_from_slice(&[255, 255, 255, 0]);
  }

  if cf == ColorFormat::I8 {
    if nema_gfx {
      data.extend(indexes.iter().map(|x| (x >> 4) | ((x & 0x0f) << 4)));
    } else {
      data.extend_from_slice(indexes);
    }
  } else {
    pack_indices(indexes, cf.bpp(), w as usize, &mut data);
  }

  LvglImage::from_data(cf, w, h, data, 0)
}

fn optimize_indexed_png_palette(
  w: u16,
  h: u16,
  palette_rgb: &[u8],
  trns: Option<&[u8]>,
  indexes: &[u8],
  nema_gfx: bool,
) -> Result<LvglImage> {
  let palette_len = palette_rgb.len() / 3;
  let mut old_palette = Vec::with_capacity(palette_len);
  for i in 0..palette_len {
    let base = i * 3;
    old_palette.push([
      palette_rgb[base],
      palette_rgb[base + 1],
      palette_rgb[base + 2],
      trns.and_then(|alpha| alpha.get(i)).copied().unwrap_or(0xff),
    ]);
  }

  let mut new_palette = Vec::<[u8; 4]>::new();
  let mut remap = [0u8; 256];
  let mut seen = [false; 256];
  for &idx in indexes {
    let idx = idx as usize;
    if seen[idx] {
      continue;
    }
    let entry = *old_palette
      .get(idx)
      .ok_or_else(|| LvglError::Format(format!("png index out of palette range: {idx}")))?;
    // Fully transparent entries are visually interchangeable in rendered output, so
    // OPTIMIZED can merge them onto the first transparent palette entry without
    // changing the visible result.
    let key = if entry[3] == 0 { [0, 0, 0, 0] } else { entry };
    let mapped = new_palette
      .iter()
      .position(|candidate| {
        let candidate_key = if candidate[3] == 0 {
          [0, 0, 0, 0]
        } else {
          *candidate
        };
        candidate_key == key
      })
      .map(|pos| pos as u8)
      .unwrap_or_else(|| {
        new_palette.push(entry);
        (new_palette.len() - 1) as u8
      });
    remap[idx] = mapped;
    seen[idx] = true;
  }

  let remapped_indexes = indexes
    .iter()
    .map(|&idx| {
      if !seen[idx as usize] {
        Err(LvglError::Format(format!(
          "png index out of palette range: {}",
          idx
        )))
      } else {
        Ok(remap[idx as usize])
      }
    })
    .collect::<Result<Vec<_>>>()?;
  let cf = indexed_cf_for_palette_len(new_palette.len());
  build_indexed_image_from_entries(w, h, cf, &new_palette, &remapped_indexes, nema_gfx)
}

struct IndexedPngMetadata {
  bit_depth: PngBitDepth,
  palette_rgb: Vec<u8>,
  trns: Option<Vec<u8>>,
}

fn parse_indexed_png_metadata(data: &[u8]) -> Result<Option<IndexedPngMetadata>> {
  if !is_png(data) {
    return Ok(None);
  }
  if data.len() < 33 {
    return Err(LvglError::Format("png is too short".to_string()));
  }
  let ihdr_len = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;
  if ihdr_len != 13 || &data[12..16] != b"IHDR" {
    return Err(LvglError::Format("png has invalid IHDR chunk".to_string()));
  }

  let bit_depth = PngBitDepth::from_u8(data[24])
    .ok_or_else(|| LvglError::Format(format!("unsupported png bit depth: {}", data[24])))?;
  if data[25] != PngColorType::Indexed as u8 {
    return Ok(None);
  }

  let mut pos = 8usize;
  let mut palette_rgb = None;
  let mut trns = None;
  while pos + 12 <= data.len() {
    let len = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
    let typ = &data[pos + 4..pos + 8];
    let chunk_data_start = pos + 8;
    let chunk_data_end = chunk_data_start
      .checked_add(len)
      .ok_or_else(|| LvglError::Format("png chunk length overflow".to_string()))?;
    let chunk_end = chunk_data_end
      .checked_add(4)
      .ok_or_else(|| LvglError::Format("png chunk length overflow".to_string()))?;
    if chunk_end > data.len() {
      return Err(LvglError::Format(
        "png chunk extends past end of file".to_string(),
      ));
    }

    match typ {
      b"PLTE" => {
        let palette = &data[chunk_data_start..chunk_data_end];
        if palette.is_empty() {
          return Err(LvglError::Format(
            "indexed png palette is empty".to_string(),
          ));
        }
        if !palette.len().is_multiple_of(3) {
          return Err(LvglError::Format(
            "indexed png palette length is not divisible by 3".to_string(),
          ));
        }
        palette_rgb = Some(palette.to_vec());
      }
      b"tRNS" => {
        trns = Some(data[chunk_data_start..chunk_data_end].to_vec());
      }
      b"IDAT" | b"IEND" => break,
      _ => {}
    }

    pos = chunk_end;
  }

  Ok(palette_rgb.map(|palette_rgb| IndexedPngMetadata {
    bit_depth,
    palette_rgb,
    trns,
  }))
}

fn finalize_image(mut image: LvglImage, options: ConvertOptions) -> Result<LvglImage> {
  if options.align == 0 {
    return Err(LvglError::Parameter("align must be at least 1".to_string()));
  }
  validate_premultiply_option(image.cf, options.premultiply)?;
  image.adjust_stride_align(options.align)?;
  if options.premultiply {
    image.premultiply()?;
  }
  Ok(image)
}

fn normalize_transparent_pixel([r, g, b, a]: [u8; 4]) -> [u8; 4] {
  // Normalize fully transparent pixels before palette quantization so invisible RGB differences
  // do not consume multiple palette entries. The Python reference does not do this, but without
  // normalization a source image may end up with several distinct "transparent colors" that are
  // visually identical.
  if a == 0 {
    [0, 0, 0, 0]
  } else {
    [r, g, b, a]
  }
}

fn validate_image_dimensions_u32(w: u32, h: u32) -> Result<()> {
  if w == 0 || h == 0 {
    return Err(LvglError::Parameter(format!(
      "image dimensions must be at least 1x1, got {w}x{h}"
    )));
  }
  if w > u16::MAX as u32 || h > u16::MAX as u32 {
    return Err(LvglError::Parameter(format!(
      "image dimensions exceed LVGL header limits: {w}x{h} (max 65535x65535)"
    )));
  }
  Ok(())
}

fn validate_image_dimensions_u16(w: u16, h: u16) -> Result<()> {
  validate_image_dimensions_u32(w as u32, h as u32)
}

fn validate_stride(stride: usize) -> Result<u16> {
  u16::try_from(stride).map_err(|_| {
    LvglError::Parameter(format!(
      "stride exceeds LVGL header limit: {stride} bytes (max 65535)"
    ))
  })
}

fn validate_premultiply_option(cf: ColorFormat, premultiply: bool) -> Result<()> {
  if premultiply && !cf.supports_premultiply() {
    return Err(LvglError::Parameter(format!(
      "premultiply not supported for {}",
      cf.name()
    )));
  }
  Ok(())
}

fn is_png(data: &[u8]) -> bool {
  data.len() >= 8 && data[..8] == [137, 80, 78, 71, 13, 10, 26, 10]
}

fn bit_extend(value: u8, bpp: usize) -> u8 {
  if value == 0 {
    return 0;
  }
  let mut res = value;
  let mut now = bpp;
  while now < 8 {
    res |= value << (8 - now);
    now += bpp;
  }
  res
}

fn color_pre_multiply(r: u8, g: u8, b: u8, a: u8, background: u32) -> (u8, u8, u8, u8) {
  let bb = (background & 0xff) as u16;
  let bg = ((background >> 8) & 0xff) as u16;
  let br = ((background >> 16) & 0xff) as u16;
  let a16 = a as u16;
  (
    ((r as u16 * a16 + (255 - a16) * br) >> 8) as u8,
    ((g as u16 * a16 + (255 - a16) * bg) >> 8) as u8,
    ((b as u16 * a16 + (255 - a16) * bb) >> 8) as u8,
    a,
  )
}

fn luma_srgb(r: u8, g: u8, b: u8) -> u8 {
  fn srgb_to_linear(x: f64) -> f64 {
    if x < 0.04045 {
      x / 12.92
    } else {
      ((x + 0.055) / 1.055).powf(2.4)
    }
  }
  fn linear_to_srgb(y: f64) -> f64 {
    if y <= 0.0031308 {
      12.92 * y
    } else {
      1.055 * y.powf(1.0 / 2.4) - 0.055
    }
  }
  let y = 0.2126 * srgb_to_linear(r as f64 / 255.0)
    + 0.7152 * srgb_to_linear(g as f64 / 255.0)
    + 0.0722 * srgb_to_linear(b as f64 / 255.0);
  (linear_to_srgb(y) * 255.0).clamp(0.0, 255.0) as u8
}

fn rgb565(r: u8, g: u8, b: u8) -> u16 {
  ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

fn unpack_rgb565(color: u16) -> [u8; 3] {
  [
    bit_extend(((color >> 11) & 0x1f) as u8, 5),
    bit_extend(((color >> 5) & 0x3f) as u8, 6),
    bit_extend((color & 0x1f) as u8, 5),
  ]
}

fn compress_wrapped(cf: ColorFormat, method: CompressMethod, data: &[u8]) -> Result<Vec<u8>> {
  if method == CompressMethod::None {
    return Ok(data.to_vec());
  }
  let compressed = match method {
    CompressMethod::None => unreachable!(),
    CompressMethod::Rle => rle_compress(data, cf.bpp().div_ceil(8)),
    CompressMethod::Lz4 => lz4_compress(data),
  };
  let mut out = Vec::with_capacity(12 + compressed.len());
  out.extend_from_slice(&(method as u32).to_le_bytes());
  out.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
  out.extend_from_slice(&(data.len() as u32).to_le_bytes());
  out.extend_from_slice(&compressed);
  Ok(out)
}

fn decompress_wrapped(data: &[u8], cf: ColorFormat) -> Result<Vec<u8>> {
  if data.len() < 12 {
    return Err(LvglError::Format(
      "compressed payload is too short".to_string(),
    ));
  }
  let method = CompressMethod::from_u32(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))?;
  let compressed_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
  let raw_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
  if data.len() < 12 + compressed_len {
    return Err(LvglError::Format(
      "compressed payload length mismatch".to_string(),
    ));
  }
  let payload = &data[12..12 + compressed_len];
  let out = match method {
    CompressMethod::None => payload.to_vec(),
    CompressMethod::Rle => rle_decompress(payload, cf.bpp().div_ceil(8), raw_len)?,
    CompressMethod::Lz4 => {
      lz4_decompress(payload, raw_len).map_err(|e| LvglError::Format(e.to_string()))?
    }
  };
  Ok(out)
}

fn rle_compress(data: &[u8], blksize: usize) -> Vec<u8> {
  let blksize = blksize.max(1);
  let mut padded = data.to_vec();
  let rem = padded.len() % blksize;
  if rem != 0 {
    padded.resize(padded.len() + blksize - rem, 0);
  }
  let mut out = Vec::new();
  let mut index = 0usize;
  while index < padded.len() {
    let repeat = repeat_count(&padded[index..], blksize);
    if repeat < 16 {
      let nonrepeat = nonrepeat_count(&padded[index..], blksize, 16);
      out.push(nonrepeat as u8 | 0x80);
      out.extend_from_slice(&padded[index..index + nonrepeat * blksize]);
      index += nonrepeat * blksize;
    } else {
      out.push(repeat as u8);
      out.extend_from_slice(&padded[index..index + blksize]);
      index += repeat * blksize;
    }
  }
  out
}

fn rle_decompress(data: &[u8], blksize: usize, raw_len: usize) -> Result<Vec<u8>> {
  let mut out = Vec::with_capacity(raw_len);
  let mut index = 0usize;
  while index < data.len() {
    let ctrl = data[index];
    index += 1;
    let count = (ctrl & 0x7f) as usize;
    if ctrl & 0x80 != 0 {
      let len = count * blksize;
      if index + len > data.len() {
        return Err(LvglError::Format("invalid RLE literal run".to_string()));
      }
      out.extend_from_slice(&data[index..index + len]);
      index += len;
    } else {
      if index + blksize > data.len() {
        return Err(LvglError::Format("invalid RLE repeated run".to_string()));
      }
      let block = &data[index..index + blksize];
      for _ in 0..count {
        out.extend_from_slice(block);
      }
      index += blksize;
    }
  }
  out.truncate(raw_len);
  Ok(out)
}

fn repeat_count(data: &[u8], blksize: usize) -> usize {
  if data.len() < blksize {
    return 0;
  }
  let first = &data[..blksize];
  let mut count = 0usize;
  for chunk in data.chunks_exact(blksize) {
    if chunk == first {
      count += 1;
      if count == 127 {
        break;
      }
    } else {
      break;
    }
  }
  count
}

fn nonrepeat_count(data: &[u8], blksize: usize, threshold: usize) -> usize {
  if data.len() < blksize {
    return 0;
  }
  let chunks: Vec<&[u8]> = data.chunks_exact(blksize).collect();
  let mut pre = chunks[0];
  let mut nonrepeat = 0usize;
  let mut repeat = 0usize;
  for chunk in chunks {
    if chunk == pre {
      repeat += 1;
      if repeat > threshold {
        // The trailing run is long enough to be a standalone repeat run; exclude
        // it so the caller can emit it separately as a more compact repeat block.
        return nonrepeat.min(127);
      }
    } else {
      pre = chunk;
      nonrepeat += 1 + repeat;
      repeat = 0;
      if nonrepeat >= 127 {
        return 127;
      }
    }
  }
  (nonrepeat + repeat).min(127)
}

fn write_c_bytes(out: &mut String, data: &[u8], stride: usize) {
  let stride = if stride == 0 { 16 } else { stride };
  for (i, byte) in data.iter().enumerate() {
    if i % stride == 0 {
      out.push_str("\n    ");
    }
    let _ = write!(out, "0x{byte:02x},");
  }
  out.push('\n');
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_c_bytes(source: &str) -> Result<Vec<u8>> {
  let bytes_re = Regex::new(r"(?i)0x([0-9a-f]{1,2})|\b(\d{1,3})\b")?;
  let map_start = source
    .find("_map[]")
    .ok_or_else(|| LvglError::Format("missing data array".to_string()))?;
  let body_start = source[map_start..]
    .find('{')
    .map(|i| map_start + i + 1)
    .ok_or_else(|| LvglError::Format("missing data array opening brace".to_string()))?;
  let body_end = source[body_start..]
    .find("};")
    .map(|i| body_start + i)
    .ok_or_else(|| LvglError::Format("missing data array closing brace".to_string()))?;
  let body = &source[body_start..body_end];
  let mut out = Vec::new();
  for caps in bytes_re.captures_iter(body) {
    let value = if let Some(hex) = caps.get(1) {
      u8::from_str_radix(hex.as_str(), 16)
        .map_err(|_| LvglError::Format("invalid byte".to_string()))?
    } else {
      caps[2]
        .parse::<u8>()
        .map_err(|_| LvglError::Format("invalid byte".to_string()))?
    };
    out.push(value);
  }
  Ok(out)
}

fn c_var_name(filename: &str) -> String {
  std::path::Path::new(filename)
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or("image")
    .replace(['-', '.'], "_")
}

const RED_THRESH: [u8; 64] = [
  1, 7, 3, 5, 0, 8, 2, 6, 7, 1, 5, 3, 8, 0, 6, 2, 3, 5, 0, 8, 2, 6, 1, 7, 5, 3, 8, 0, 6, 2, 7, 1,
  0, 8, 2, 6, 1, 7, 3, 5, 8, 0, 6, 2, 7, 1, 5, 3, 2, 6, 1, 7, 3, 5, 0, 8, 6, 2, 7, 1, 5, 3, 8, 0,
];

const GREEN_THRESH: [u8; 64] = [
  1, 3, 2, 2, 3, 1, 2, 2, 2, 2, 0, 4, 2, 2, 4, 0, 3, 1, 2, 2, 1, 3, 2, 2, 2, 2, 4, 0, 2, 2, 0, 4,
  1, 3, 2, 2, 3, 1, 2, 2, 2, 2, 0, 4, 2, 2, 4, 0, 3, 1, 2, 2, 1, 3, 2, 2, 2, 2, 4, 0, 2, 2, 0, 4,
];

const BLUE_THRESH: [u8; 64] = [
  5, 3, 8, 0, 6, 2, 7, 1, 3, 5, 0, 8, 2, 6, 1, 7, 8, 0, 6, 2, 7, 1, 5, 3, 0, 8, 2, 6, 1, 7, 3, 5,
  6, 2, 7, 1, 5, 3, 8, 0, 2, 6, 1, 7, 3, 5, 0, 8, 7, 1, 5, 3, 8, 0, 6, 2, 1, 7, 3, 5, 0, 8, 2, 6,
];
