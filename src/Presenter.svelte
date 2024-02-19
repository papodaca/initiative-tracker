<script>
  import { onMount } from "svelte";
  import { listen } from '@tauri-apps/api/event'
  import { WebviewWindow } from "@tauri-apps/api/window"
  import PlayerList from "./components/PlayerList.svelte";
  import { getState } from "./store"
  let state = {};

  const incomingState = async (s) => {
    state = s
    const currentImage = state[state.currentCampaign].images.find(i => i.active)
    if (currentImage) {
      document.body.setAttribute("style", `--bg-image: url('${currentImage.fileUrl}')`)
    }
  }
  onMount(() => {
    getState().then(incomingState)
    listen('state-change', (event) => incomingState(event.payload))
  })
  const onKeyUp = async (e) => {
    let presenter = WebviewWindow.getByLabel("presenter")
    if (presenter == null) return
    let fullscreen = await presenter.isFullscreen()
    if (e.key === "F11") {
      presenter.setFullscreen(!fullscreen)
    } else if (fullscreen && e.key === "Escape") {
      presenter.setFullscreen(false)
    }
  }
</script>

<svelte:window on:keyup={onKeyUp} />

{#if state[state.currentCampaign] && state[state.currentCampaign].players && state[state.currentCampaign].initiativeVisible}
  <div style="font-size: {state.dislaySize.toString()}em">
    <PlayerList
      players={state[state.currentCampaign].players}
      initiative={true}
      enemyHealthVisible={state[state.currentCampaign].healthVisible}
      healthVisible={state[state.currentCampaign].enemyHealthVisible} />
  </div>
{/if}
