import "bootstrap/dist/css/bootstrap.min.css"
import "@fortawesome/fontawesome-free/css/fontawesome.min.css"
import "@fortawesome/fontawesome-free/css/regular.min.css"
import "@fortawesome/fontawesome-free/css/solid.min.css"

import "./presenter.css"
import Presenter from "./Presenter.svelte"

const presenter = new Presenter({
  target: document.getElementById("app")
})

export default presenter
