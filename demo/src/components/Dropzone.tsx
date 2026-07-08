import { createSignal } from 'solid-js'

interface DropzoneProps {
  accept?: string
  onFile: (file: File) => void
  label?: string
}

export default function Dropzone(props: DropzoneProps) {
  const [dragover, setDragover] = createSignal(false)
  let inputRef: HTMLInputElement | undefined

  const handleDrop = (e: DragEvent) => {
    e.preventDefault()
    setDragover(false)
    const file = e.dataTransfer?.files[0]
    if (file) props.onFile(file)
  }

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault()
    setDragover(true)
  }

  const handleDragLeave = () => setDragover(false)

  const handleClick = () => inputRef?.click()

  const handleChange = (e: Event) => {
    const input = e.target as HTMLInputElement
    const file = input.files?.[0]
    if (file) props.onFile(file)
    input.value = ''
  }

  return (
    <div
      class={`dropzone ${dragover() ? 'dragover' : ''}`}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onClick={handleClick}
    >
      <input
        ref={(el) => {
          inputRef = el
        }}
        type="file"
        accept={props.accept}
        onChange={handleChange}
        style={{ display: 'none' }}
      />
      <p class="dropzone-text">
        <strong>Click to browse</strong> or drag and drop
        <br />
        <span style={{ 'font-size': '0.85rem', 'margin-top': '0.25rem', display: 'inline-block' }}>
          {props.label || 'Select a file'}
        </span>
      </p>
    </div>
  )
}
