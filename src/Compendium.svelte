<script>
  import { onMount } from "svelte";

  import { type as osType } from "@tauri-apps/plugin-os"

  import { loadDB, getModels } from "./db"
  import renderers from "./renderers"

  let models = {}, loading = true, loadProgress = 0, currentModel = "", showPopup = false, currentPlatform, searchTerm, searchResults = []

  let tabs = [
    { id: "armor", name: "Armor", active: true },
    { id: "classes", name: "Classes", active: false },
    { id: "magicitems", name: "Magic Items", active: false },
    { id: "monsters", name: "Monsters", active: false },
    { id: "races", name: "Races", active: false },
    { id: "spells", name: "Spells", active: false },
    { id: "weapons", name: "Weapons", active: false },
  ]

  const activeTab = () => tabs.filter(tab => tab.active)[0]

  let currentTab = activeTab()

  const loadCurrentTab = async () => {
    if (models[currentTab.id] != null) return
    models[currentTab.id] = await getModels(currentTab.id)
  }

  const selectTab = (tabId) => {
    return async (event) => {
      tabs.forEach(tab => {
        tab.active = false
        if (tab.id == tabId) tab.active = true
      });
      currentTab = activeTab()
      loading = true
      await loadCurrentTab()
      loading = false
    }
  }

  const handleDatumClick = (datum, set) => {
    return async (event) => {
      models[datum._model] = models[datum._model].map(dat => {
        if(dat.slug == datum.slug) {
          dat._clicked = set
        } else {
          dat.clicked = false
        }
        return dat
      })
    }
  }

  const reloadDB = (force) => {
    return async (event) => {
      loading = true
      loadProgress = 0
      if (force) models = {}
      await loadDB(force, (model, progress) => { currentModel = model; loadProgress = progress })
      await loadCurrentTab()
      loading = false
    }
  }
  onMount(reloadDB(false))
  onMount(async () => currentPlatform = await osType())

  const isCommand = (event) => ((currentPlatform === "Darwin" && event.metaKey) || (currentPlatform !== "Darwin" && event.ctrlKey))

  const onKeyUp = (event) => {
    if (isCommand(event) && event.key === "k") {
      showPopup = true
      //set style="overflow: hidden; padding-right: 17px;"
      //class="modal-open"
    } else if (event.key === "Escape") {
      showPopup = false
      searchResults = []
      searchTerm = null
    }
  }

  const onSearch = (event) => {
    if (searchTerm) {
      searchResults = monsters.filter(m => m.slug.includes(searchTerm))
    }
  }
</script>

<style>
  div.modal.fade.show {
    display: block;
  }
</style>

<svelte:window on:keyup={onKeyUp} />

<svelte:body class:modal-open={showPopup} />

{#if showPopup}
<div class="modal fade show" id="search_modal" tabindex="-1" aria-labelledby="search monsers" aria-hidden="true">
  <div class="modal-dialog modal-sm">
    <div class="modal-body">
      <form on:submit|preventDefault>
        <input type="search" class="form-control" bind:value={searchTerm}/>
        <ul>
          {#each searchResults as monster}
            <li>{monster.slug}</li>
          {/each}
        </ul>
      </form>
    </div>
  </div>
</div>

{/if}

<button class="btn btn-primary" on:click={reloadDB(true)} disabled={loading}>
  {#if loading}
    <span class="spinner-border spinner-border-sm" aria-hidden="true"></span>
    <span role="status">Loading {currentModel}...</span>
  {:else}
    <i class="fa-solid fa-rotate-right"></i>&nbsp;Reload Database
  {/if}
</button><br/>

{#if loading}
  <div class="progress" role="progressbar" aria-label="Loading progress" aria-valuenow={loadProgress} aria-valuemin="0" aria-valuemax="100">
    <div class="progress-bar" style="width: {loadProgress}%">{loadProgress}%</div>
  </div>
{/if}

<ul class="nav nav-tabs">
  {#each tabs as tab}
  <li class="nav-item">
    <a class="nav-link" class:active={currentTab.id == tab.id} aria-current="page" href="#" on:click|preventDefault={selectTab(tab.id)}>{tab.name}</a>
  </li>
  {/each}
</ul>

{#if !loading}
<ul>
{#each models[currentTab.id] as datum}
  <li>{datum.name}&nbsp;{datum._clicked}
  {#if datum._clicked}
    <button class="btn btn-sm btn-primary" on:click={handleDatumClick(datum, false)}>Close</button>
    <svelte:component this={renderers[datum._model]} datum={datum} />
  {:else}
    <button class="btn btn-sm btn-primary" on:click={handleDatumClick(datum, true)}>View</button>
  {/if}
  </li>
{/each}
</ul>
{/if}

{#if showPopup}
<div class="modal-backdrop fade show"></div>
{/if}