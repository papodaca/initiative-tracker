<script>
  import { onMount } from "svelte"

  import { WebviewWindow } from "@tauri-apps/api/webviewWindow"
  import { listen } from '@tauri-apps/api/event'

  import PlayerList from "./components/PlayerList.svelte"
  import ImageList from "./components/ImageList.svelte"
  import { getState, saveStore, setState as setStoreState } from "./store"
  import { toTitleCase } from "./utils"

  const DEFAULT_HEALTH = 10

  let state = $state({})
  let presenterFullscreen = $state(false)
  let newCampaignName = $state()
  let presenter
  let presenterVisible = $state(false)
  let appWindow

  appWindow = WebviewWindow.getCurrent()

  const openPresenter = async () => {
    if (presenter == null) return
    if (!presenterVisible) {
      presenter.show()
      presenterVisible = true
    }
  }

  const togglePresenterFullscreen = async () => {
    presenterFullscreen = !presenterFullscreen
    presenter.emit('set-fullscreen', { fullscreen: presenterFullscreen })
  }
  listen('fullscreen', (event) => { presenterFullscreen = event.payload.fullscreen })

  const closePresenter = async (event) => {
    event?.preventDefault?.()
    if (presenter == null) return
    await presenter.hide()
    presenterVisible = false
  }

  appWindow.onCloseRequested(async () => {
    await saveStore()
    presenter.destroy()
    appWindow.destroy()
  })

  const loadState = async () => {
    presenter = await WebviewWindow.getByLabel("presenter");
    presenterVisible = await presenter.isVisible()
    presenter.onCloseRequested(closePresenter)
    state = await getState()
    if(state == null) state = {}
    let changed = false
    if (state.dislaySize == null) {
      state.dislaySize = 1.0
      changed = true
    }
    if (state.currentCampaign == null) {
      state.currentCampaign = "default"
      state.campaigns = [state.currentCampaign]
      changed = true
    }
    if (state[state.currentCampaign] == null) {
      state[state.currentCampaign] = defaultCampaing()
      changed = true
    }
    if (changed) broadcastState()
  }
  const defaultCampaing = () => ({
    players: [
      {
        id: crypto.randomUUID(),
        name: "Player 1",
        health: DEFAULT_HEALTH,
        maxHealth: DEFAULT_HEALTH,
        initiative: 3
      },{
        id: crypto.randomUUID(),
        name: "Player 2",
        health: DEFAULT_HEALTH,
        maxHealth: DEFAULT_HEALTH,
        initiative: 2
      },{
        id: crypto.randomUUID(),
        name: "Player 3",
        health: DEFAULT_HEALTH,
        maxHealth: DEFAULT_HEALTH,
        initiative: 1
      }
    ],
    images: []
  })
  onMount(() => loadState())
  const setSate = () => {
    updateCampaign(defaultCampaing())
  }
  const playersChange = (players) => {
    updateCampaign({
      players
    }, false)
    sortList()
    broadcastState()
  }
  const sortList = () => {
    updateCampaign({
      players: state[state.currentCampaign].players.sort((a, b) => Number(b.initiative) - Number(a.initiative))
    })
  }

  const addCampaign = (_el) => {
    state = {
      ...state,
      campaigns: [...state.campaigns, newCampaignName],
      [newCampaignName]: defaultCampaing()
    }
    newCampaignName = undefined
    broadcastState()
  }
  const addPlayer = (kind) =>((_e) => {
    updateCampaign({
      players: [
        ...state[state.currentCampaign].players,
        {
          id: crypto.randomUUID(),
          name: `New ${toTitleCase(kind)}`,
          initiative: 0,
          health: DEFAULT_HEALTH,
          maxHealth: DEFAULT_HEALTH,
          kind
        }
      ]
    }, false)
  })
  const clearMonsters = (_e) => {
    updateCampaign({
      players: state[state.currentCampaign].players.filter(p => p.kind !== 'monster')
    })
  }
  const updatePlayerActive = () => {
    updateCampaign({
      players: state[state.currentCampaign].players.map((p, i) => {
          if (i === state[state.currentCampaign].currentPlayer) {
            p.active = true
          } else {
            p.active = false
          }
          return p
        })
    })
  }
  const startInitiative = (_e) => {
    state[state.currentCampaign].currentPlayer = 0
    updatePlayerActive()
    broadcastState()
  }
  const nextPlayer = (_e) => {
    state[state.currentCampaign].currentPlayer += 1
    if (state[state.currentCampaign].currentPlayer >= state[state.currentCampaign].players.length) {
      state[state.currentCampaign].currentPlayer = 0
    }
    updatePlayerActive()
    broadcastState()

  }
  const previousPlayer = (_e) => {
    state[state.currentCampaign].currentPlayer -= 1
    if (state[state.currentCampaign].currentPlayer < 0) {
      state[state.currentCampaign].currentPlayer = state[state.currentCampaign].players.length - 1
    }
    updatePlayerActive()
    broadcastState()
  }
  const endInitiative = (_e) => {
    state[state.currentCampaign].currentPlayer = null
    updatePlayerActive()
    broadcastState()
  }

  const updateCampaign = (data, update = true) => {
    state = {
      ...state,
      [state.currentCampaign]: {
        ...state[state.currentCampaign],
        ...data
      }
    }
    if (update) broadcastState()
  }

  const toggle = (field) => ((_e) => {
    updateCampaign({
      [field]: !state[state.currentCampaign][field]
    })
    broadcastState()
  })
  const broadcastState = () => setStoreState(state)
  const imagesChange = (images) => {
    updateCampaign({
      images
    })
  }
  const initiateRest = (kind) => (() => {
    if (kind === 'long') {
      updateCampaign({
        players: state[state.currentCampaign].players.map(p => {
          if (p.kind === 'player' || p.kind === 'npc') p.health = p.maxHealth
          return p
        })
      })
    }
  })
</script>
<style>
  .display {
    max-width: fit-content;
  }
  select.form-control {
    max-width: fit-content;
    display: inline-block;
  }
  input.form-control {
    max-width: 150px;
    display: inline-block;
  }
  button.btn {
    margin-bottom: 5px;
  }
  button.btn[disabled],
  button.btn:disabled {
    cursor: not-allowed;
  }
</style>
Campaign:&nbsp;
<select class="form-control" bind:value={state.currentCampaign} onchange={broadcastState}>
  {#each state.campaigns || [] as campaign}
    <option value={campaign}>{toTitleCase(campaign)}</option>
  {/each}
</select>
<input placeholder="Name" type="text" class="form-control" bind:value={newCampaignName}/>
<button class="btn btn-success" onclick={addCampaign} disabled={newCampaignName == null || newCampaignName.length < 1}>
  <i class="fa-regular fa-square-plus"></i>&nbsp;Add Campaign
</button>
<br/>
<button class="btn btn-primary" onclick={openPresenter} disabled={presenterVisible}>
  <i class="fa-solid fa-arrow-up-right-from-square"></i>&nbsp;Open Presenter
</button>
<button class="btn btn-primary" onclick={togglePresenterFullscreen} disabled={!presenterVisible}>
  {#if presenterFullscreen}
    <i class="fa-solid fa-minimize"></i>&nbsp;Presenter
  {:else}
    <i class="fa-solid fa-expand"></i>&nbsp;Presenter
  {/if}
</button>
<button class="btn btn-danger" onclick={closePresenter} disabled={!presenterVisible}>
  <i class="fa-solid fa-circle-xmark"></i>&nbsp;Presenter
</button>
<button class="btn btn-primary" onclick={setSate}>
  <i class="fa-solid fa-rotate-right"></i>&nbsp;Set Default
</button><br/>
<button class="btn btn-success" onclick={addPlayer('player')}>
  <i class="fa-regular fa-square-plus"></i>&nbsp;Add Player
</button>
<button class="btn btn-info" onclick={addPlayer('npc')}>
  <i class="fa-regular fa-square-plus"></i>&nbsp;Add NPC
</button>
<button class="btn btn-danger" onclick={addPlayer('monster')}>
  <i class="fa-regular fa-square-plus"></i>&nbsp;Add Monster
</button>
<button class="btn btn-danger" onclick={clearMonsters}>
  <i class="fa-solid fa-square-xmark"></i>&nbsp;Clear Monsters
</button><br/>
<button class="btn btn-primary" onclick={startInitiative}>
  <i class="fa-solid fa-play"></i>&nbsp;Start
</button>
<button class="btn btn-primary" onclick={nextPlayer}>
  <i class="fa-solid fa-arrow-down-long"></i>&nbsp;Next
</button>
<button class="btn btn-primary" onclick={previousPlayer}>
  <i class="fa-solid fa-arrow-up-long"></i>&nbsp;Previous
</button>
<button class="btn btn-primary" onclick={endInitiative}>
  <i class="fa-solid fa-hand"></i>&nbsp;End
</button><br/>
<button class="btn btn-primary" onclick={toggle('initiativeVisible')}>
  {#if state[state.currentCampaign] && state[state.currentCampaign].initiativeVisible}
    <i class="fa-solid fa-eye-slash"></i>
  {:else}
    <i class="fa-solid fa-eye"></i>
  {/if}&nbsp;Initiative
</button>
<button class="btn btn-primary" onclick={toggle('healthVisible')}>
  {#if state[state.currentCampaign] && state[state.currentCampaign].healthVisible}
    <i class="fa-solid fa-eye-slash"></i>
  {:else}
    <i class="fa-solid fa-eye"></i>
  {/if}&nbsp;Enemy Health
</button>
<button class="btn btn-primary" onclick={toggle('enemyHealthVisible')}>
  {#if state[state.currentCampaign] && state[state.currentCampaign].enemyHealthVisible}
    <i class="fa-solid fa-eye-slash"></i>
  {:else}
    <i class="fa-solid fa-eye"></i>
  {/if}&nbsp;Player Health
</button>
<button class="btn btn-primary" onclick={initiateRest('long')}>
  <i class="fa-solid fa-bed"></i>&nbsp;Long Rest
</button><br/>
Dsiplay Size: <input type="range" min="1" max="5" step="0.1" bind:value={state.dislaySize} onchange={broadcastState} />&nbsp;{state.dislaySize}
<div class="display">
  <PlayerList
    players={state[state.currentCampaign] && state[state.currentCampaign].players}
    onupdate={playersChange}
    initiative={false}
    healthVisible={true}
    enemyHealthVisible={true} />
</div>
<ImageList
  images={state[state.currentCampaign] && state[state.currentCampaign].images}
  onupdate={imagesChange} />
