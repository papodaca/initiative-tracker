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

  const incomingState = async (s) => {
    state = s
    applyTheme(state.theme)
    const currentImage = state[state.currentCampaign].images.find(i => i.active)
    if (currentImage) {
      document.body.setAttribute("style", `--bg-image: url('${currentImage.fileUrl}')`)
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
