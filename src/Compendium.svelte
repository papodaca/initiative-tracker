<script>
  import { onMount } from "svelte";

  import { type as osType } from "@tauri-apps/plugin-os"

  import { loadDB, getMonsters } from "./db"

  let monsters = [], loading = true, loadProgress = 0, showPopup = false, currentPlatform, searchTerm, searchResults = []

  const reloadDB = (force) => {
    return async (event) => {
      loading = true
      loadProgress = 0
      monsters = []
      await loadDB(force, (progress) => loadProgress = progress)
      monsters = await getMonsters()
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
    <span role="status">Loading...</span>
  {:else}
    <i class="fa-solid fa-rotate-right"></i>&nbsp;Reload Database
  {/if}
</button><br/>

{#if loading}
  <div class="progress" role="progressbar" aria-label="Loading progress" aria-valuenow={loadProgress} aria-valuemin="0" aria-valuemax="100">
    <div class="progress-bar" style="width: {loadProgress}%">{loadProgress}%</div>
  </div>
{/if}

<ul>
{#each monsters as monster}
  <li>{monster.slug}</li>
{/each}
</ul>

{#if showPopup}
<div class="modal-backdrop fade show"></div>
{/if}