<script>
  import { onMount } from "svelte"

  import { WebviewWindow } from "@tauri-apps/api/webviewWindow"
  import { listen } from '@tauri-apps/api/event'

  import PlayerList from "./components/PlayerList.svelte"
  import ImageList from "./components/ImageList.svelte"
  import SettingsOverlay from "./components/SettingsOverlay.svelte"
  import AddCampaignDialog from "./components/AddCampaignDialog.svelte"
  import { getState, saveStore, setState as setStoreState } from "./store"
  import { toTitleCase } from "./utils"
  import { applyTheme, watchSystemTheme } from "./theme"

  const DEFAULT_HEALTH = 10

  let state = $state({})
  let presenterFullscreen = $state(false)
  let settingsOpen = $state(false)
  let addCampaignOpen = $state(false)
  let presenter
  let presenterVisible = $state(false)
  let appWindow
  let stopSystemWatcher = null

  // Add-combatant drawer form fields.
  let newName = $state('')
  let newInitiative = $state('')
  let newMaxHealth = $state('')

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

  // Dead is derived from HP (health <= 0), not authored. Used to normalize
  // legacy saved state on load and to keep it consistent on player changes.
  const deriveDead = (player) => {
    const dead = Number(player.health) <= 0
    const changed = !!player.dead !== dead
    player.dead = dead
    return changed
  }
  const normalizeDead = (players) => (players || []).some(deriveDead)

  const loadState = async () => {
    presenter = await WebviewWindow.getByLabel("presenter");
    presenterVisible = await presenter.isVisible()
    presenter.onCloseRequested(closePresenter)
    state = await getState()
    if(state == null) state = {}
    let changed = false
    if (state.theme == null) {
      state.theme = "system"
      changed = true
    }
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
    ;(state.campaigns || []).forEach(c => {
      if (state[c] == null) return
      if (state[c].showInitiativeRoll == null) { state[c].showInitiativeRoll = true; changed = true }
      if (state[c].autoHideInactive == null) { state[c].autoHideInactive = false; changed = true }
    })
    if ((state.campaigns || []).some(c => normalizeDead(state[c] && state[c].players))) changed = true
    if (changed) broadcastState()
    applyTheme(state.theme)
    stopSystemWatcher?.()
    if (state.theme === "system") {
      stopSystemWatcher = watchSystemTheme(() => applyTheme("system"))
    }
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
    images: [],
    showInitiativeRoll: true,
    autoHideInactive: false
  })
  onMount(() => loadState())
  const playersChange = (players) => {
    normalizeDead(players)
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

  const addCampaign = (name) => {
    if (!name || !name.trim() || (state.campaigns || []).includes(name)) return
    state = {
      ...state,
      campaigns: [...(state.campaigns || []), name],
      [name]: defaultCampaing()
    }
    addCampaignOpen = false
    broadcastState()
  }
  const openAddCampaign = () => { addCampaignOpen = true }
  const closeAddCampaign = () => { addCampaignOpen = false }

  const addCombatant = (kind) => (() => {
    const maxHealth = Number(newMaxHealth) || DEFAULT_HEALTH
    const player = {
      id: crypto.randomUUID(),
      name: (newName || '').trim() || `New ${toTitleCase(kind)}`,
      initiative: Number(newInitiative) || 0,
      health: maxHealth,
      maxHealth,
      kind,
      dead: maxHealth <= 0
    }
    updateCampaign({
      players: [...(state[state.currentCampaign].players || []), player]
    })
    newName = ''
    newInitiative = ''
    newMaxHealth = ''
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

  const openSettings = () => { settingsOpen = true }
  const closeSettings = () => { settingsOpen = false }
  const saveSettings = ({ name }) => {
    // Rename current campaign (rekey) when the name changed to a unique value.
    if (name && name !== state.currentCampaign &&
        (state.campaigns || []).includes(state.currentCampaign) &&
        !(state.campaigns || []).includes(name)) {
      const oldName = state.currentCampaign
      const campaign = state[oldName]
      const campaigns = (state.campaigns || []).map(c => c === oldName ? name : c)
      const { [oldName]: _removed, ...rest } = state
      state = { ...rest, campaigns, currentCampaign: name, [name]: campaign }
    }
    broadcastState()
    applyTheme(state.theme)
    stopSystemWatcher?.()
    stopSystemWatcher = state.theme === "system" ? watchSystemTheme(() => applyTheme("system")) : null
    settingsOpen = false
  }

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
  .console {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    background-color: var(--color-surface);
    border-bottom: 1px solid var(--color-edge);
    padding: 12px 16px;
  }
  .header-top {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .header-top .campaign-select { flex: 1; min-width: 0; }

  .content-scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .content-scroll > :global(*) {
    flex-shrink: 0;
  }

  details {
    background-color: var(--color-surface);
    border: 1px solid var(--color-edge);
    border-radius: 8px;
    overflow: hidden;
  }
  summary {
    padding: 12px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--color-muted);
    cursor: pointer;
    font-weight: 600;
    user-select: none;
  }
  details[open] summary { border-bottom: 1px solid var(--color-edge); }
  .drawer-content {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .presenter-row { display: flex; gap: 8px; }

  .visibility-row {
    display: flex;
    justify-content: space-between;
    gap: 6px;
  }

  .quick-add-form { display: flex; flex-direction: column; gap: 8px; }
  .form-row { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .btn-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }

  .combat-sticky-footer {
    background-color: var(--color-surface);
    border-top: 1px solid var(--color-edge);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .main-loop-buttons { display: flex; gap: 8px; }
  .btn-xl { flex: 2; padding: 14px; font-size: 15px; font-weight: bold; }
  .secondary-buttons {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
  }
</style>

<div class="console">
  <header>
    <div class="header-top">
      <select class="form-control campaign-select" bind:value={state.currentCampaign} onchange={broadcastState}>
        {#each state.campaigns || [] as campaign}
          <option value={campaign}>{toTitleCase(campaign)}</option>
        {/each}
      </select>
      <button class="btn btn-info btn-sm" onclick={openAddCampaign} title="Add campaign" aria-label="add campaign">
        <i class="fa-regular fa-square-plus"></i>
      </button>
      <button class="btn btn-primary btn-sm" onclick={openSettings} title="Settings">
        <i class="fa-solid fa-gear"></i>&nbsp;Settings
      </button>
    </div>
  </header>

  <div class="content-scroll">
    <details open>
      <summary><i class="fa-solid fa-display"></i>&nbsp;Presenter &amp; Media</summary>
      <div class="drawer-content">
        <button class="btn btn-primary" onclick={openPresenter} disabled={presenterVisible}>
          <i class="fa-solid fa-arrow-up-right-from-square"></i>&nbsp;Open Presenter Window
        </button>
        <div class="presenter-row">
          <button class="btn btn-info" onclick={togglePresenterFullscreen} disabled={!presenterVisible}>
            {#if presenterFullscreen}
              <i class="fa-solid fa-minimize"></i>&nbsp;Exit Fullscreen
            {:else}
              <i class="fa-solid fa-expand"></i>&nbsp;Fullscreen
            {/if}
          </button>
          <button class="btn btn-danger" onclick={closePresenter} disabled={!presenterVisible}>
            <i class="fa-solid fa-circle-xmark"></i>&nbsp;Close
          </button>
        </div>
        <ImageList
          images={state[state.currentCampaign] && state[state.currentCampaign].images}
          onupdate={imagesChange} />
      </div>
    </details>

    <details>
      <summary><i class="fa-regular fa-square-plus"></i>&nbsp;Add Combatant</summary>
      <div class="drawer-content">
        <div class="quick-add-form">
          <input type="text" class="form-control" placeholder="Name" bind:value={newName} />
          <div class="form-row">
            <input type="number" class="form-control" placeholder="Init Roll" bind:value={newInitiative} />
            <input type="number" class="form-control" placeholder="Max HP" bind:value={newMaxHealth} />
          </div>
          <div class="btn-grid">
            <button class="btn btn-success" onclick={addCombatant('player')}>PC</button>
            <button class="btn btn-primary" onclick={addCombatant('npc')}>NPC</button>
            <button class="btn btn-danger" onclick={addCombatant('monster')}>Monster</button>
          </div>
        </div>
      </div>
    </details>

    <div class="visibility-row">
      <button class="btn btn-info btn-sm" onclick={toggle('initiativeVisible')} title="Toggle initiative visibility">
        <i class="fa-solid fa-eye{state[state.currentCampaign] && state[state.currentCampaign].initiativeVisible ? '' : '-slash'}"></i>&nbsp;Initiative
      </button>
      <button class="btn btn-info btn-sm" onclick={toggle('healthVisible')} title="Toggle enemy HP visibility">
        <i class="fa-solid fa-eye{state[state.currentCampaign] && state[state.currentCampaign].healthVisible ? '' : '-slash'}"></i>&nbsp;Enemy HP
      </button>
      <button class="btn btn-info btn-sm" onclick={toggle('enemyHealthVisible')} title="Toggle player HP visibility">
        <i class="fa-solid fa-eye{state[state.currentCampaign] && state[state.currentCampaign].enemyHealthVisible ? '' : '-slash'}"></i>&nbsp;Player HP
      </button>
    </div>

    <PlayerList
      players={state[state.currentCampaign] && state[state.currentCampaign].players}
      onupdate={playersChange}
      initiative={false}
      healthVisible={true}
      enemyHealthVisible={true} />
  </div>

  <footer class="combat-sticky-footer">
    <div class="main-loop-buttons">
      <button class="btn btn-info" onclick={previousPlayer} title="Previous turn" aria-label="previous turn">
        <i class="fa-solid fa-backward-step"></i>
      </button>
      <button class="btn btn-primary btn-xl" onclick={nextPlayer}>
        <i class="fa-solid fa-forward-step"></i>&nbsp;NEXT TURN
      </button>
    </div>
    <div class="secondary-buttons">
      <button class="btn btn-info btn-sm" onclick={startInitiative}>
        <i class="fa-solid fa-play"></i>&nbsp;Start
      </button>
      <button class="btn btn-info btn-sm" onclick={endInitiative}>
        <i class="fa-solid fa-hand"></i>&nbsp;End
      </button>
      <button class="btn btn-info btn-sm" onclick={initiateRest('long')}>
        <i class="fa-solid fa-bed"></i>&nbsp;Long Rest
      </button>
      <button class="btn btn-danger btn-sm" onclick={clearMonsters}>
        <i class="fa-solid fa-skull"></i>&nbsp;Clear Monsters
      </button>
    </div>
  </footer>
</div>

{#if settingsOpen}
  <SettingsOverlay {state} onsave={saveSettings} onclose={closeSettings} />
{/if}
{#if addCampaignOpen}
  <AddCampaignDialog onconfirm={addCampaign} onclose={closeAddCampaign} />
{/if}
