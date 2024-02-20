<script>
  import { onMount } from "svelte";
  import { listen, emit } from '@tauri-apps/api/event'
  import { WebviewWindow } from "@tauri-apps/api/window"
  import PlayerList from "./components/PlayerList.svelte";
  import { getState } from "./store"
    import { event } from "@tauri-apps/api";
  let state = {}, fullscreenState = false;

  const incomingState = async (s) => {
    state = s
    const currentImage = state[state.currentCampaign].images.find(i => i.active)
    if (currentImage) {
      document.body.setAttribute("style", `--bg-image: url('${currentImage.fileUrl}')`)
    }
  }
  const setFullscreen = async (fullscreen) => {
    fullscreenState = fullscreen
    let presenter = WebviewWindow.getByLabel("presenter")
    if (presenter == null) return
    await presenter.setFullscreen(fullscreen)
    emit('fullscreen', { fullscreen })
  }
  onMount(() => {
    getState().then(incomingState)
    listen('state-change', (event) => incomingState(event.payload))
    listen('set-fullscreen', (event) => setFullscreen(event.payload.fullscreen))
  })
  const onKeyUp = async (event) => {
    if (event.key === "F11") {
      setFullscreen(!fullscreenState)
    } else if (fullscreenState && event.key === "Escape") {
      setFullscreen(false)
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
