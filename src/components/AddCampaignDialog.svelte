<script>
  let { onconfirm, onclose } = $props()
  let name = $state('')

  const onBackdrop = (e) => { if (e.target === e.currentTarget) onclose?.() }
  const onKeydown = (e) => { if (e.key === 'Escape') onclose?.() }
  const confirm = () => {
    if (name && name.trim()) onconfirm?.(name.trim())
  }
</script>

<div class="dialog-overlay" onclick={onBackdrop} onkeydown={onKeydown} tabindex="-1" role="dialog" aria-modal="true" aria-label="Add campaign">
  <div class="dialog-container">
    <div class="dialog-header">
      <div class="dialog-title">Add Campaign</div>
      <button class="dialog-close-btn" aria-label="close" onclick={onclose}>×</button>
    </div>
    <input type="text" class="form-control" placeholder="Campaign name" bind:value={name} />
    <div class="dialog-actions">
      <button class="btn btn-info" onclick={onclose}>Cancel</button>
      <button class="btn btn-success" onclick={confirm} disabled={!name || !name.trim()}>Create</button>
    </div>
  </div>
</div>

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  .dialog-container {
    background-color: var(--color-surface);
    border: 1px solid var(--color-edge);
    border-radius: 8px;
    width: 90%;
    max-width: 300px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 15px;
  }
  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--color-edge);
    padding-bottom: 10px;
  }
  .dialog-title { font-weight: 600; font-size: 16px; }
  .dialog-close-btn {
    background: transparent;
    border: none;
    color: var(--color-muted);
    font-size: 18px;
    cursor: pointer;
  }
  .dialog-actions { display: flex; justify-content: flex-end; gap: 8px; }
</style>
