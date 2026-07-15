<script>
  import { onMount } from "svelte"

  import { listen, emit } from '@tauri-apps/api/event'
  import { WebviewWindow } from "@tauri-apps/api/webviewWindow"

  import PlayerList from "./components/PlayerList.svelte"
  import { getState } from "./store"
  import { applyTheme } from "./theme"

  let state = $state({})
  let fullscreenState = false
  let presenter = WebviewWindow.getCurrent()

  let campaign = $derived(state[state.currentCampaign])
  // Auto-Hide Inactive Turns removes dead combatants from the Presenter view.
  let visiblePlayers = $derived(
    campaign && campaign.players
      ? (campaign.autoHideInactive ? campaign.players.filter(p => !p.dead) : campaign.players)
      : []
  )
  // Show Initiative Roll toggles the roll number on the Presenter.
  let showInitiativeRoll = $derived(campaign ? campaign.showInitiativeRoll !== false : true)

  // Background transition state
  let bg1 = $state("")
  let bg2 = $state("")
  let activeLayer = $state(1)
  let lastUrl = ""

  const preloadAndTransition = (url) => {
    if (url === lastUrl) return
    lastUrl = url

    const img = new Image()
    img.src = url
    img.onload = () => {
      if (lastUrl !== url) return
      if (activeLayer === 1) {
        bg2 = url
        activeLayer = 2
      } else {
        bg1 = url
        activeLayer = 1
      }
    }
    img.onerror = () => {
      if (lastUrl !== url) return
      if (activeLayer === 1) {
        bg2 = url
        activeLayer = 2
      } else {
        bg1 = url
        activeLayer = 1
      }
    }
  }

  const incomingState = async (s) => {
    state = s
    applyTheme(state.theme)
    const currentCampaign = state[state.currentCampaign]
    const currentImage = currentCampaign && currentCampaign.images ? currentCampaign.images.find(i => i.active) : null
    if (currentImage) {
      preloadAndTransition(currentImage.fileUrl)
    } else {
      lastUrl = ""
      bg1 = ""
      bg2 = ""
    }
  }
  const setFullscreen = async (fullscreen) => {
    fullscreenState = fullscreen
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

<svelte:window onkeyup={onKeyUp} />

<div class="presenter-bg-container">
  <div class="presenter-bg-layer" style="background-image: {bg1 ? `url('${bg1}')` : 'none'}; opacity: {activeLayer === 1 ? 1 : 0}"></div>
  <div class="presenter-bg-layer" style="background-image: {bg2 ? `url('${bg2}')` : 'none'}; opacity: {activeLayer === 2 ? 1 : 0}"></div>
</div>

{#if campaign && campaign.players && campaign.initiativeVisible}
  <div style="font-size: {state.dislaySize.toString()}em">
    <PlayerList
      players={visiblePlayers}
      initiative={true}
      showInitiativeRoll={showInitiativeRoll}
      enemyHealthVisible={campaign.healthVisible}
      healthVisible={campaign.enemyHealthVisible} />
  </div>
{/if}
