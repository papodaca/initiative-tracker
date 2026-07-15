<script>
  import { untrack } from "svelte"
  let { state: stateValue, onsave, onclose } = $props()
  // Campaign name is a key, not a field, so it is edited as a local draft and
  // rekeyed on save by the parent (KTD6).
  let name = $state(untrack(() => stateValue.currentCampaign))
  const themes = ["system", "light", "dark"]
  const title = (t) => t[0].toUpperCase() + t.slice(1)

  const onBackdrop = (e) => { if (e.target === e.currentTarget) onclose?.() }
  const onKeydown = (e) => { if (e.key === 'Escape') onclose?.() }
  const save = () => onsave?.({ name })
</script>

<div class="settings-overlay" onclick={onBackdrop} onkeydown={onKeydown} tabindex="-1" role="dialog" aria-modal="true" aria-label="Campaign settings">
  <div class="settings-container">
    <div class="settings-header">
      <div class="settings-title">Campaign Settings</div>
      <button class="settings-close-btn" aria-label="close settings" onclick={onclose}>×</button>
    </div>

    <div class="setting-item">
      <span class="setting-label">Theme</span>
      <div class="setting-control">
        <select class="form-control" bind:value={stateValue.theme}>
          {#each themes as t}
            <option value={t}>{title(t)}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="setting-item">
      <span class="setting-label">Display Size</span>
      <div class="setting-control">
        <input type="range" min="1" max="5" step="0.1" bind:value={stateValue.dislaySize} />
      </div>
    </div>

    <div class="setting-item">
      <span class="setting-label">Campaign Name</span>
      <div class="setting-control">
        <input type="text" class="form-control" bind:value={name} />
      </div>
    </div>

    <div class="setting-item">
      <span class="setting-label">Show Initiative Roll</span>
      <div class="setting-control">
        <input type="checkbox" bind:checked={stateValue[stateValue.currentCampaign].showInitiativeRoll} />
      </div>
    </div>

    <div class="setting-item">
      <span class="setting-label">Auto-Hide Inactive Turns</span>
      <div class="setting-control">
        <input type="checkbox" bind:checked={stateValue[stateValue.currentCampaign].autoHideInactive} />
      </div>
    </div>

    <button class="btn btn-success" onclick={save}>Save Changes</button>
  </div>
</div>

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  .settings-container {
    background-color: var(--color-surface);
    border: 1px solid var(--color-edge);
    border-radius: 8px;
    width: 90%;
    max-width: 320px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 15px;
  }
  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--color-edge);
    padding-bottom: 10px;
  }
  .settings-title { font-weight: 600; font-size: 16px; }
  .settings-close-btn {
    background: transparent;
    border: none;
    color: var(--color-muted);
    font-size: 18px;
    cursor: pointer;
  }
  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .setting-label { font-size: 14px; }
  .setting-control { display: flex; align-items: center; }
  .setting-control select,
  .setting-control input[type="text"] { min-width: 130px; }
</style>
