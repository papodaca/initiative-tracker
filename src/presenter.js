import { mount } from 'svelte'
import './app.css'
import '@fortawesome/fontawesome-free/css/all.min.css'
import './presenter.css'
import { applyTheme } from './theme'
import Presenter from './Presenter.svelte'

applyTheme('system')

const app = mount(Presenter, {
  target: document.getElementById('app')
})

export default app
