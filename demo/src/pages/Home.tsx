import { createResource } from 'solid-js'
import { Marked } from 'marked'
import { markedHighlight } from 'marked-highlight'
import hljs from 'highlight.js/lib/core'
import javascript from 'highlight.js/lib/languages/javascript'
import bash from 'highlight.js/lib/languages/bash'
import c from 'highlight.js/lib/languages/c'
import readme from '../README.md?raw'

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('js', javascript)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('c', c)

const marked = new Marked(
  markedHighlight({
    langPrefix: 'hljs language-',
    highlight(code, lang) {
      if (lang && hljs.getLanguage(lang)) {
        return hljs.highlight(code, { language: lang }).value
      }
      return code
    },
  }),
)

async function renderReadme(): Promise<string> {
  return marked.parse(readme) as string
}

export default function Home() {
  const [html] = createResource(renderReadme)

  return (
    <div>
      {html.loading && (
        <div style={{ 'text-align': 'center', padding: '3rem' }}>
          <div class="loading-spinner" />
        </div>
      )}
      {html() && <div class="markdown-body" innerHTML={html()} />}
    </div>
  )
}
