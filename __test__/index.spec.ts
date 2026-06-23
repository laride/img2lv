import test from 'ava'
import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

import { imageToBin, imageToC, lvglToPng, lvglToRgba, lvglWidth, lvglHeight } from '../index'

const IMG_DIR = join(dirname(fileURLToPath(import.meta.url)), 'images')
const isWasiBinding = process.env.NAPI_RS_FORCE_WASI != null || (process.arch as string) === 'wasm32'
const cArrayDecodeTest = isWasiBinding ? test.skip : test

function loadTestImage(name: string): Buffer {
  return readFileSync(join(IMG_DIR, name))
}

function parseLvglHeader(bin: Buffer) {
  return {
    cf: bin[1] & 0x1f,
    flags: bin.readUInt16LE(2),
    width: bin.readUInt16LE(4),
    height: bin.readUInt16LE(6),
    stride: bin.readUInt16LE(8),
    data: bin.subarray(12),
  }
}

function unpackRowIndices(data: Buffer, width: number, bpp: number, stride: number): number[] {
  if (bpp === 8) {
    return [...data.subarray(0, width)]
  }

  const out: number[] = []
  const mask = (1 << bpp) - 1
  for (const byte of data.subarray(0, stride)) {
    for (let shift = 8 - bpp; shift >= 0; shift -= bpp) {
      out.push((byte >> shift) & mask)
      if (out.length === width) {
        return out
      }
    }
  }
  return out
}

function normalizeTransparentRgba(data: Buffer): number[] {
  const out = [...data]
  for (let i = 0; i < out.length; i += 4) {
    if (out[i + 3] === 0) {
      out[i] = 0
      out[i + 1] = 0
      out[i + 2] = 0
    }
  }
  return out
}

test('imageToBin produces valid LVGL binary header', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(png, 'ARGB8888', 0, 1, false, 'NONE')
  t.is(bin[0], 0x19, 'magic byte should be 0x19')
  t.is(bin[1], 0x10, 'cf byte should be ARGB8888 (0x10)')
  t.true(bin.length > 12, 'output should contain header + data')
})

test('imageToBin with RGB565 format', (t) => {
  const jpg = loadTestImage('kalen-emsley-Bkci_8qcdvQ-unsplash-small.jpg')
  const bin = imageToBin(jpg, 'RGB565', 0, 1, false, 'NONE')
  t.is(bin[0], 0x19)
  t.is(bin[1], 0x12, 'cf byte should be RGB565 (0x12)')
})

test('imageToBin with indexed I8 format', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(png, 'I8', 0, 1, false, 'NONE')
  t.is(bin[0], 0x19)
  t.is(bin[1], 0x0a, 'cf byte should be I8 (0x0A)')
})

test('imageToBin with AUTO format selects indexed', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(png, 'AUTO', 0, 1, false, 'NONE')
  t.is(bin[0], 0x19)
  const cf = bin[1] & 0x1f
  t.true([0x07, 0x08, 0x09, 0x0a].includes(cf), `cf should be indexed format, got 0x${cf.toString(16)}`)
})

test('imageToBin with LZ4 compression', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(png, 'ARGB8888', 0, 1, false, 'LZ4')
  t.is(bin[0], 0x19)
  t.truthy(bin[2] & 0x08, 'compressed flag should be set')
})

test('imageToBin with RLE compression', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(png, 'ARGB8888', 0, 1, false, 'RLE')
  t.is(bin[0], 0x19)
  t.truthy(bin[2] & 0x08, 'compressed flag should be set')
})

test('roundtrip: bin -> png', (t) => {
  const original = loadTestImage('kalen-emsley-Bkci_8qcdvQ-unsplash-small.jpg')
  const bin = imageToBin(original, 'ARGB8888', 0, 1, false, 'NONE')
  const pngOut = lvglToPng(bin, false)
  t.true(pngOut.length > 0, 'output PNG should not be empty')
  t.is(pngOut[0], 0x89, 'output should start with PNG magic')
  t.is(pngOut[1], 0x50, 'P')
  t.is(pngOut[2], 0x4e, 'N')
  t.is(pngOut[3], 0x47, 'G')
})

test('roundtrip: bin -> rgba', (t) => {
  const original = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(original, 'ARGB8888', 0, 1, false, 'NONE')
  const w = lvglWidth(bin, false)
  const h = lvglHeight(bin, false)
  t.true(w > 0)
  t.true(h > 0)
  const rgba = lvglToRgba(bin, false)
  t.is(rgba.length, w * h * 4, 'RGBA data should have 4 bytes per pixel')
})

test('roundtrip: compressed bin -> png', (t) => {
  const original = loadTestImage('lucas-calloch-P-yzuyWFEIk-unsplash-small.webp')
  for (const compress of ['RLE', 'LZ4'] as const) {
    const bin = imageToBin(original, 'ARGB8888', 0, 1, false, compress)
    const pngOut = lvglToPng(bin, false)
    t.true(pngOut.length > 0, `${compress} roundtrip should produce PNG`)
  }
})

test('lvglWidth and lvglHeight return correct dimensions', (t) => {
  const original = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(original, 'ARGB8888', 0, 1, false, 'NONE')
  const w = lvglWidth(bin, false)
  const h = lvglHeight(bin, false)
  t.true(w > 0 && w < 10000)
  t.true(h > 0 && h < 10000)
})

test('imageToC produces valid C source', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const cSource = imageToC(png, 'test.png', null, 'ARGB8888', 0, 1, false, 'NONE')
  t.true(cSource.includes('LV_COLOR_FORMAT_ARGB8888'))
  t.true(cSource.includes('lv_image_dsc_t'))
  t.true(cSource.includes('_map[]'))
})

test('imageToC with custom output name', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const cSource = imageToC(png, 'test.png', 'my_icon', 'RGB565', 0, 1, false, 'NONE')
  t.true(cSource.includes('my_icon_map[]'))
  t.true(cSource.includes('lv_image_dsc_t my_icon'))
})

cArrayDecodeTest('C array roundtrip exposes correct dimensions', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(png, 'ARGB8888', 0, 1, false, 'NONE')
  const cSource = imageToC(png, 'test.png', null, 'ARGB8888', 0, 1, false, 'NONE')
  const cBuffer = Buffer.from(cSource, 'utf8')

  t.is(lvglWidth(cBuffer, true), lvglWidth(bin, false))
  t.is(lvglHeight(cBuffer, true), lvglHeight(bin, false))
})

cArrayDecodeTest('C array roundtrip decodes to PNG', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const cSource = imageToC(png, 'test.png', null, 'RGB565', 0, 1, false, 'NONE')
  const pngOut = lvglToPng(Buffer.from(cSource, 'utf8'), true)

  t.true(pngOut.length > 0, 'output PNG should not be empty')
  t.is(pngOut[0], 0x89, 'output should start with PNG magic')
  t.is(pngOut[1], 0x50, 'P')
  t.is(pngOut[2], 0x4e, 'N')
  t.is(pngOut[3], 0x47, 'G')
})

cArrayDecodeTest('compressed C array decodes to RGBA like the binary path', (t) => {
  const original = loadTestImage('lucas-calloch-P-yzuyWFEIk-unsplash-small.webp')
  const bin = imageToBin(original, 'ARGB8888', 0, 1, false, 'LZ4')
  const cSource = imageToC(original, 'test.webp', null, 'ARGB8888', 0, 1, false, 'LZ4')

  const rgbaFromBin = lvglToRgba(bin, false)
  const rgbaFromC = lvglToRgba(Buffer.from(cSource, 'utf8'), true)

  t.deepEqual([...rgbaFromC], [...rgbaFromBin], 'C array and binary decode should match')
})

test('imageToBin with premultiply', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(png, 'ARGB8888', 0, 1, true, 'NONE')
  t.is(bin[0], 0x19)
  t.truthy(bin[2] & 0x01, 'premultiplied flag should be set')
})

test('imageToBin rejects premultiply for unsupported formats before conversion', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const error = t.throws(() => imageToBin(png, 'XRGB8888', 0, 1, true, 'NONE'))
  t.regex(error.message, /premultiply not supported for XRGB8888/i)
})

test('indexed PNG fast path preserves palette and indices when capacity allows', (t) => {
  const png = loadTestImage('fixture-indexed-trns-preserve.png')
  const bin = imageToBin(png, 'I2', 0, 1, false, 'NONE')
  const { cf, width, height, stride, data } = parseLvglHeader(bin)

  t.is(cf, 0x08, 'cf byte should be I2 (0x08)')
  t.is(width, 2)
  t.is(height, 2)
  t.is(stride, 1)
  t.deepEqual(
    [...data.subarray(0, 16)],
    [0x00, 0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0x80, 0x03, 0x02, 0x01, 0x00, 0xfc, 0xfb, 0xfa, 0x00],
    'palette bytes should preserve the original PNG palette and tRNS values',
  )
  t.deepEqual([...data.subarray(16)], [0x10, 0xb0], 'packed pixel indices should be preserved')
})

test('indexed PNG fast path still applies align and premultiply options', (t) => {
  const png = loadTestImage('fixture-indexed-trns-preserve.png')
  const bin = imageToBin(png, 'I2', 0, 4, true, 'NONE')
  const { flags, stride, data } = parseLvglHeader(bin)

  t.truthy(flags & 0x01, 'premultiplied flag should be set')
  t.is(stride, 4, 'stride should respect the requested alignment')
  t.deepEqual(
    [...data.subarray(0, 8)],
    [0x00, 0x00, 0xff, 0xff, 0x00, 0x80, 0x00, 0x80],
    'palette entries should be premultiplied in the fast path',
  )
})

test('premultiplied indexed decode returns the same RGBA as non-premultiplied decode', (t) => {
  const png = loadTestImage('fixture-indexed-trns-preserve.png')
  const normal = imageToBin(png, 'I2', 0, 1, false, 'NONE')
  const premultiplied = imageToBin(png, 'I2', 0, 1, true, 'NONE')

  const rgbaNormal = lvglToRgba(normal, false)
  const rgbaPremultiplied = lvglToRgba(premultiplied, false)

  t.deepEqual(
    normalizeTransparentRgba(rgbaPremultiplied),
    normalizeTransparentRgba(rgbaNormal),
    'un-premultiplication during decode should preserve visible RGBA while normalizing fully transparent RGB to 0',
  )
})

test('AUTO preserves indexed PNG palette sizing like the Python reference', (t) => {
  const png = loadTestImage('fixture-indexed-optimized-shrink.png')
  const bin = imageToBin(png, 'AUTO', 0, 1, false, 'NONE')
  const { cf, stride, data } = parseLvglHeader(bin)

  t.is(cf, 0x08, 'AUTO should keep the original 4-entry palette tier (I2)')
  t.is(stride, 1)
  t.deepEqual(
    [...data.subarray(0, 16)],
    [0x00, 0x00, 0xff, 0xff, 0x00, 0xff, 0x00, 0xff, 0x03, 0x02, 0x01, 0x00, 0xfc, 0xfb, 0xfa, 0x00],
    'AUTO should preserve the source indexed palette bytes',
  )
})

test('OPTIMIZED shrinks indexed PNG palette when a smaller tier is losslessly possible', (t) => {
  const png = loadTestImage('fixture-indexed-optimized-shrink.png')
  const bin = imageToBin(png, 'OPTIMIZED', 0, 1, false, 'NONE')
  const { cf, width, height, stride, data } = parseLvglHeader(bin)

  t.is(cf, 0x07, 'OPTIMIZED should shrink to I1 after collapsing transparent variants')
  t.is(width, 2)
  t.is(height, 2)
  t.is(stride, 1)
  t.deepEqual(
    [...data.subarray(0, 8)],
    [0x00, 0x00, 0xff, 0xff, 0x03, 0x02, 0x01, 0x00],
    'OPTIMIZED should keep the first transparent entry and drop redundant palette entries',
  )
  t.deepEqual([...data.subarray(8)], [0x40, 0x80], 'pixel indices should be remapped into the smaller palette')
})

test('OPTIMIZED keeps original indexed palette when compression does not change the tier', (t) => {
  const png = loadTestImage('fixture-indexed-trns-preserve.png')
  const autoBin = imageToBin(png, 'AUTO', 0, 1, false, 'NONE')
  const optimizedBin = imageToBin(png, 'OPTIMIZED', 0, 1, false, 'NONE')

  t.deepEqual(
    [...optimizedBin],
    [...autoBin],
    'OPTIMIZED should fall back to the preserved palette when no smaller tier exists',
  )
})

test('transparent RGB variants collapse to one transparent palette entry during quantization', (t) => {
  const png = loadTestImage('fixture-transparent-variants.png')
  const bin = imageToBin(png, 'I8', 0, 1, false, 'NONE')
  const { data } = parseLvglHeader(bin)
  const palette = data.subarray(0, 256 * 4)
  const indices = unpackRowIndices(data.subarray(256 * 4), 8, 8, 8)

  const transparentIndices = new Set(indices.filter((index) => palette[index * 4 + 3] === 0))

  t.is(transparentIndices.size, 1, 'all fully transparent pixels should reuse one palette entry')
})

test('invalid color format throws', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  t.throws(() => imageToBin(png, 'INVALID_FORMAT', 0, 1, false, 'NONE'))
})

test('invalid compression method throws', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  t.throws(() => imageToBin(png, 'ARGB8888', 0, 1, false, 'ZSTD'))
})

test('imageToBin rejects stride alignment that overflows LVGL header stride', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const error = t.throws(() => imageToBin(png, 'ARGB8888', 0, 70_000, false, 'NONE'))
  t.regex(error.message, /stride exceeds LVGL header limit/i)
})

test('lvglToPng rejects zero-width LVGL images', (t) => {
  const bin = Buffer.alloc(12 + 4)
  bin[0] = 0x19
  bin[1] = 0x10
  bin.writeUInt16LE(0, 2)
  bin.writeUInt16LE(0, 4)
  bin.writeUInt16LE(1, 6)
  bin.writeUInt16LE(4, 8)

  const error = t.throws(() => lvglToPng(bin, false))
  t.regex(error.message, /image dimensions must be at least 1x1/i)
})
