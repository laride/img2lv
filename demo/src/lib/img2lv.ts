export type { ConvertOptions, DecodeOptions } from 'img2lv'

type Img2LvModule = typeof import('img2lv')

let mod: Img2LvModule | null = null
let initPromise: Promise<Img2LvModule> | null = null

export function ensureInit(): Promise<Img2LvModule> {
  if (!initPromise) {
    initPromise = import('img2lv').then((m) => {
      mod = m
      return m
    })
  }
  return initPromise
}

export function getModule(): Img2LvModule {
  if (!mod) throw new Error('img2lv not initialized — call ensureInit() first')
  return mod
}

export const COLOR_FORMATS = [
  'AUTO',
  'OPTIMIZED',
  'ARGB8888',
  'XRGB8888',
  'RGB888',
  'RGB565',
  'RGB565_SWAPPED',
  'RGB565A8',
  'ARGB8565',
  'ARGB8888_PREMULTIPLIED',
  'AL88',
  'L8',
  'A8',
  'A4',
  'A2',
  'A1',
  'I8',
  'I4',
  'I2',
  'I1',
] as const

export const COMPRESS_METHODS = ['NONE', 'RLE', 'LZ4'] as const
