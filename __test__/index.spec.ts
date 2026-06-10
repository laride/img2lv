import test from 'ava'
import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

import { imageToBin, imageToC, lvglToPng, lvglToRgba, lvglWidth, lvglHeight } from '../index'

const IMG_DIR = join(dirname(fileURLToPath(import.meta.url)), 'images')

function loadTestImage(name: string): Buffer {
  return readFileSync(join(IMG_DIR, name))
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

test('imageToBin with premultiply', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  const bin = imageToBin(png, 'ARGB8888', 0, 1, true, 'NONE')
  t.is(bin[0], 0x19)
  t.truthy(bin[2] & 0x01, 'premultiplied flag should be set')
})

test('invalid color format throws', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  t.throws(() => imageToBin(png, 'INVALID_FORMAT', 0, 1, false, 'NONE'))
})

test('invalid compression method throws', (t) => {
  const png = loadTestImage('simon-twukN12EN7c-unsplash-small.png')
  t.throws(() => imageToBin(png, 'ARGB8888', 0, 1, false, 'ZSTD'))
})
