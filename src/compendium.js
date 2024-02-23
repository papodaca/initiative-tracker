import "bootstrap/dist/css/bootstrap.min.css"
import "@fortawesome/fontawesome-free/css/fontawesome.min.css"
import "@fortawesome/fontawesome-free/css/regular.min.css"
import "@fortawesome/fontawesome-free/css/solid.min.css"

import "./presenter.css"
import Compendium from "./Compendium.svelte"

const compendium = new Compendium({
  target: document.getElementById("app")
})
