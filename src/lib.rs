#![deny(clippy::all)]

mod convert;

use convert::{ColorFormat, CompressMethod, ConvertOptions};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn image_to_bin(
  input: Buffer,
  cf: String,
  background: u32,
  align: u32,
  premultiply: bool,
  compress: String,
) -> Result<Buffer> {
  let options = ConvertOptions {
    cf: ColorFormat::parse(&cf).map_err(to_napi_err)?,
    background,
    align: align as usize,
    premultiply,
    compress: CompressMethod::parse(&compress).map_err(to_napi_err)?,
    rgb565_dither: false,
    nema_gfx: false,
  };
  let result = convert::image_to_lvgl_bin(&input, options).map_err(to_napi_err)?;
  Ok(result.into())
}

#[napi]
pub fn image_to_c(
  input: Buffer,
  input_name: String,
  output_name: Option<String>,
  cf: String,
  background: u32,
  align: u32,
  premultiply: bool,
  compress: String,
) -> Result<String> {
  let options = ConvertOptions {
    cf: ColorFormat::parse(&cf).map_err(to_napi_err)?,
    background,
    align: align as usize,
    premultiply,
    compress: CompressMethod::parse(&compress).map_err(to_napi_err)?,
    rgb565_dither: false,
    nema_gfx: false,
  };
  convert::image_to_lvgl_c(&input, &input_name, output_name.as_deref(), options)
    .map_err(to_napi_err)
}

#[napi]
pub fn lvgl_to_png(input: Buffer, is_c_array: bool) -> Result<Buffer> {
  let result = convert::lvgl_to_png(&input, is_c_array).map_err(to_napi_err)?;
  Ok(result.into())
}

#[napi]
pub fn lvgl_to_rgba(input: Buffer, is_c_array: bool) -> Result<Buffer> {
  let result = convert::lvgl_to_rgba(&input, is_c_array).map_err(to_napi_err)?;
  Ok(result.into())
}

#[napi]
pub fn lvgl_width(input: Buffer, is_c_array: bool) -> Result<u32> {
  convert::lvgl_width(&input, is_c_array)
    .map(|v| v as u32)
    .map_err(to_napi_err)
}

#[napi]
pub fn lvgl_height(input: Buffer, is_c_array: bool) -> Result<u32> {
  convert::lvgl_height(&input, is_c_array)
    .map(|v| v as u32)
    .map_err(to_napi_err)
}

fn to_napi_err(err: convert::LvglError) -> Error {
  Error::from_reason(err.to_string())
}
