import { mount } from 'svelte'
import './app.css'
import '@fortawesome/fontawesome-free/css/all.min.css'
import { applyTheme } from './theme'
import Console from './Console.svelte'

applyTheme('system')

const app = mount(Console, {
  target: document.getElementById('app')
})

export default app
