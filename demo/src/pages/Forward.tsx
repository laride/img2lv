import { createSignal, Show } from 'solid-js'
import Dropzone from '../components/Dropzone'
import { ensureInit, getModule, COLOR_FORMATS, COMPRESS_METHODS } from '../lib/img2lv'
import type { ConvertOptions } from '../lib/img2lv'

export default function Forward() {
  const [file, setFile] = createSignal<File | null>(null)
  const [preview, setPreview] = createSignal<string>('')
  const [cf, setCf] = createSignal('OPTIMIZED')
  const [compress, setCompress] = createSignal('NONE')
  const [outputMode, setOutputMode] = createSignal<'bin' | 'c'>('bin')
  const [align, setAlign] = createSignal(1)
  const [premultiply, setPremultiply] = createSignal(false)
  const [dither, setDither] = createSignal(false)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal('')
  const [result, setResult] = createSignal<{
    data: Uint8Array
    text?: string
    size: number
  } | null>(null)

  const handleFile = (f: File) => {
    setFile(f)
    setError('')
    setResult(null)
    const url = URL.createObjectURL(f)
    setPreview(url)
  }

  const convert = async () => {
    const f = file()
    if (!f) return

    setLoading(true)
    setError('')
    setResult(null)

    try {
      await ensureInit()
      const { imageToBin, imageToC } = getModule()
      const arrayBuf = await f.arrayBuffer()
      const input = new Uint8Array(arrayBuf) as any

      const options: ConvertOptions = {
        cf: cf() as ConvertOptions['cf'],
        compress: compress() as ConvertOptions['compress'],
        align: align(),
        premultiply: premultiply(),
        rgb565Dither: dither(),
      }

      if (outputMode() === 'bin') {
        const bin = imageToBin(input, options)
        const data = new Uint8Array(bin.slice())
        setResult({ data, size: data.byteLength })
      } else {
        const cSource = imageToC(input, f.name, null, options)
        const encoded = new TextEncoder().encode(cSource)
        setResult({ data: encoded, text: cSource, size: encoded.length })
      }
    } catch (e: any) {
      setError(e?.message || String(e))
    } finally {
      setLoading(false)
    }
  }

  const download = () => {
    const r = result()
    if (!r) return
    const ext = outputMode() === 'bin' ? '.bin' : '.c'
    const baseName = file()?.name?.replace(/\.[^.]+$/, '') || 'output'
    const blob = new Blob([new Uint8Array(r.data)], { type: 'application/octet-stream' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = baseName + ext
    a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div>
      <div class="page-header">
        <h1 class="page-title">Image → LVGL v9 Image Format</h1>
        <p class="page-desc">
          Convert standard image files (PNG, JPEG, WebP, BMP) to LVGL v9 binary or C source format.
        </p>
      </div>

      <div class="card">
        <Dropzone accept="image/*" onFile={handleFile} label="PNG, JPEG, WebP, BMP, etc." />

        <Show when={preview()}>
          <div class="preview-container">
            <img src={preview()} alt="Preview" class="preview-image" />
          </div>
        </Show>
      </div>

      <div class="card" style={{ 'margin-top': '1rem' }}>
        <div class="options-grid">
          <div class="form-group">
            <label class="form-label">Color Format</label>
            <select class="form-select" value={cf()} onInput={(e) => setCf(e.currentTarget.value)}>
              {COLOR_FORMATS.map((fmt) => (
                <option value={fmt}>{fmt}</option>
              ))}
            </select>
          </div>

          <div class="form-group">
            <label class="form-label">Compression</label>
            <select class="form-select" value={compress()} onInput={(e) => setCompress(e.currentTarget.value)}>
              {COMPRESS_METHODS.map((m) => (
                <option value={m}>{m}</option>
              ))}
            </select>
          </div>

          <div class="form-group">
            <label class="form-label">Output Format</label>
            <select
              class="form-select"
              value={outputMode()}
              onInput={(e) => setOutputMode(e.currentTarget.value as 'bin' | 'c')}
            >
              <option value="bin">Binary (.bin)</option>
              <option value="c">C Source (.c)</option>
            </select>
          </div>

          <div class="form-group">
            <label class="form-label">Stride Align</label>
            <input
              class="form-input"
              type="number"
              min="1"
              value={align()}
              onInput={(e) => setAlign(parseInt(e.currentTarget.value) || 1)}
            />
          </div>
        </div>

        <div style={{ display: 'flex', gap: '1.5rem', 'margin-top': '0.5rem' }}>
          <label class="form-checkbox">
            <input type="checkbox" checked={premultiply()} onChange={(e) => setPremultiply(e.currentTarget.checked)} />
            Premultiply Alpha
          </label>
          <label class="form-checkbox">
            <input type="checkbox" checked={dither()} onChange={(e) => setDither(e.currentTarget.checked)} />
            RGB565 Dither
          </label>
        </div>
      </div>

      <div style={{ 'margin-top': '1.25rem', display: 'flex', gap: '0.75rem' }}>
        <button class="btn btn-primary" onClick={convert} disabled={!file() || loading()}>
          {loading() && <span class="loading-spinner" />}
          Convert
        </button>
        <Show when={result()}>
          <button class="btn btn-secondary" onClick={download}>
            Download
          </button>
        </Show>
      </div>

      <Show when={error()}>
        <div class="status-msg error">{error()}</div>
      </Show>

      <Show when={result()}>
        <div class="output-area">
          <div class="output-header">
            <span class="output-title">Output</span>
            <span class="output-stats">{(result()!.size / 1024).toFixed(2)} KB</span>
          </div>
          <Show
            when={result()!.text}
            fallback={
              <p style={{ color: 'var(--text-muted)', 'font-size': '0.9rem' }}>Binary output ready for download.</p>
            }
          >
            <pre class="output-code">{result()!.text}</pre>
          </Show>
        </div>
      </Show>
    </div>
  )
}
