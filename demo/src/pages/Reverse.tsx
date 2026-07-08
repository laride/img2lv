import { Buffer } from 'buffer'
import { createSignal, Show } from 'solid-js'
import Dropzone from '../components/Dropzone'
import { ensureInit, getModule } from '../lib/img2lv'

export default function Reverse() {
  const [file, setFile] = createSignal<File | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal('')
  const [result, setResult] = createSignal<{
    url: string
    width: number
    height: number
    size: number
  } | null>(null)

  const handleFile = (f: File) => {
    setFile(f)
    setError('')
    if (result()) {
      URL.revokeObjectURL(result()!.url)
    }
    setResult(null)
  }

  const convert = async () => {
    const f = file()
    if (!f) return

    setLoading(true)
    setError('')
    if (result()) {
      URL.revokeObjectURL(result()!.url)
    }
    setResult(null)

    try {
      await ensureInit()
      const { lvglToPng, lvglWidth, lvglHeight } = getModule()
      const arrayBuf = await f.arrayBuffer()
      const input = Buffer.from(arrayBuf)

      const width = lvglWidth(input)
      const height = lvglHeight(input)
      const pngBuf = lvglToPng(input)
      const pngBytes = new Uint8Array(pngBuf.buffer.byteLength === pngBuf.byteLength ? pngBuf : pngBuf.slice())

      const blob = new Blob([pngBytes], { type: 'image/png' })
      const url = URL.createObjectURL(blob)

      setResult({ url, width, height, size: pngBytes.byteLength })
    } catch (e: any) {
      setError(e?.message || String(e))
    } finally {
      setLoading(false)
    }
  }

  const download = () => {
    const r = result()
    if (!r) return
    const baseName = file()?.name?.replace(/\.[^.]+$/, '') || 'output'
    const a = document.createElement('a')
    a.href = r.url
    a.download = baseName + '.png'
    a.click()
  }

  return (
    <div>
      <div class="page-header">
        <h1 class="page-title">LVGL v9 Image Format → PNG</h1>
        <p class="page-desc">Convert LVGL v9 binary data back to a standard PNG image.</p>
      </div>

      <div class="card">
        <Dropzone accept=".bin,.lvgl" onFile={handleFile} label="LVGL binary file (.bin)" />

        <Show when={file()}>
          <div style={{ 'margin-top': '1rem', 'font-size': '0.9rem', color: 'var(--text-secondary)' }}>
            Selected: <strong>{file()!.name}</strong> ({(file()!.size / 1024).toFixed(2)} KB)
          </div>
        </Show>
      </div>

      <div style={{ 'margin-top': '1.25rem', display: 'flex', gap: '0.75rem' }}>
        <button class="btn btn-primary" onClick={convert} disabled={!file() || loading()}>
          {loading() && <span class="loading-spinner" />}
          Convert
        </button>
        <Show when={result()}>
          <button class="btn btn-secondary" onClick={download}>
            Download PNG
          </button>
        </Show>
      </div>

      <Show when={error()}>
        <div class="status-msg error">{error()}</div>
      </Show>

      <Show when={result()}>
        <div class="output-area">
          <div class="output-header">
            <span class="output-title">Result</span>
            <span class="output-stats">
              {result()!.width}×{result()!.height} &middot; {(result()!.size / 1024).toFixed(2)} KB
            </span>
          </div>
          <div class="preview-container">
            <img src={result()!.url} alt="Converted PNG" class="preview-image" />
          </div>
        </div>
      </Show>
    </div>
  )
}
