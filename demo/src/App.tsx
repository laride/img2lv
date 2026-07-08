import { Router, Route } from '@solidjs/router'
import Layout from './components/Layout'
import Home from './pages/Home'
import Forward from './pages/Forward'
import Reverse from './pages/Reverse'

export default function App() {
  return (
    <Router base={import.meta.env.BASE_URL.replace(/\/$/, '')} root={Layout}>
      <Route path="/" component={Home} />
      <Route path="/forward" component={Forward} />
      <Route path="/reverse" component={Reverse} />
    </Router>
  )
}
