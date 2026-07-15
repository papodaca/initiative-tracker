import { mount } from 'svelte'
import 'bootstrap/dist/css/bootstrap.min.css'
import '@fortawesome/fontawesome-free/css/all.min.css'
import './presenter.css'
import Presenter from './Presenter.svelte'

const app = mount(Presenter, {
  target: document.getElementById('app')
})

export default app
