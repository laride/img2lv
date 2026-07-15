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
