import { mount } from 'svelte'
import 'bootstrap/dist/css/bootstrap.css'
import '@fortawesome/fontawesome-free/css/all.min.css'
import Console from './Console.svelte'

const app = mount(Console, {
  target: document.getElementById('app')
})

export default app
