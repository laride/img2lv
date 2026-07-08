/* @refresh reload */
import { Buffer } from 'buffer'
;(globalThis as typeof globalThis & { Buffer: typeof Buffer }).Buffer = Buffer

import { render } from 'solid-js/web'
import App from './App'
import './styles.css'

const root = document.getElementById('root')

render(() => <App />, root!)
