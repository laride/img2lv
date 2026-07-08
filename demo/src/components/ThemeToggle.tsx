import { createSignal, onMount } from 'solid-js'

export default function ThemeToggle() {
  const [dark, setDark] = createSignal(false)

  onMount(() => {
    const stored = localStorage.getItem('img2lv-theme')
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    const isDark = stored ? stored === 'dark' : prefersDark
    setDark(isDark)
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light')
  })

  const toggle = () => {
    const next = !dark()
    setDark(next)
    document.documentElement.setAttribute('data-theme', next ? 'dark' : 'light')
    localStorage.setItem('img2lv-theme', next ? 'dark' : 'light')
  }

  return (
    <button class="theme-toggle" onClick={toggle} title={dark() ? 'Switch to light mode' : 'Switch to dark mode'}>
      {dark() ? '☀️' : '🌙'}
    </button>
  )
}
