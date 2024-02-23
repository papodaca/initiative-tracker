import "bootstrap/dist/css/bootstrap.min.css"
import "@fortawesome/fontawesome-free/css/fontawesome.min.css"
import "@fortawesome/fontawesome-free/css/regular.min.css"
import "@fortawesome/fontawesome-free/css/solid.min.css"

import Console from "./Console.svelte"

const console = new Console({
  target: document.getElementById("app")
})
