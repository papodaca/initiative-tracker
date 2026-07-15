<script>
  // based on https://github.com/brunomolteni/svelte-sortable-list/
  import { quintOut } from "svelte/easing";
  import { crossfade } from "svelte/transition";
  import { flip } from "svelte/animate";
  import InPlaceEdit from "./InPlaceEdit.svelte";

  // PROPS
  let { players = [], initiative = true, sortable = false, healthVisible = false, enemyHealthVisible = false, showInitiativeRoll = true, onupdate } = $props()

  // FLIP ANIMATION
  const [send, receive] = crossfade({
    duration: d => Math.sqrt(d * 200),

    fallback(node, params) {
      const style = getComputedStyle(node);
      const transform = style.transform === "none" ? "" : style.transform;

      return {
        duration: 600,
        easing: quintOut,
        css: t => `
					transform: ${transform} scale(${t});
					opacity: ${t}
				`
      };
    }
  });

  // DRAG AND DROP
  let isOver = $state(false);
  const getDraggedParent = node =>
    node.dataset && node.dataset.index
      ? node.dataset
      : getDraggedParent(node.parentNode);
  const start = ev => {
    ev.dataTransfer.setData("source", ev.target.dataset.index);
  };
  const over = ev => {
    ev.preventDefault();
    let dragged = getDraggedParent(ev.target);
    if (isOver !== dragged.id) isOver = dragged.id;
  };
  const leave = ev => {
    let dragged = getDraggedParent(ev.target);
    if (isOver === dragged.id) isOver = false;
  };
  const drop = ev => {
    isOver = false;
    ev.preventDefault();
    let dragged = getDraggedParent(ev.target);
    let from = ev.dataTransfer.getData("source");
    let to = dragged.index;
    reorder({ from, to });
  };

  // Dead is derived from HP: a combatant is dead when health <= 0, revives on HP > 0.
  const deriveDead = (player) => { player.dead = Number(player.health) <= 0 }

  const updateField = (id, field) => {
    return (event) => {
      if (field === 'health' || field === 'maxHealth') {
        const player = players.find(p => p.id === id)
        if (player) deriveDead(player)
      }
      onupdate?.(players);
    }
  }
  const deletePlayer = (id) => ((_event) => {
    const newList = players.filter(p => p.id !== id)
    onupdate?.(newList)
  })

  const KIND_LABEL = { player: 'PC', npc: 'NPC', monster: 'Monster' }
  const kindLabel = (kind) => KIND_LABEL[kind] || (kind ? kind[0].toUpperCase() + kind.slice(1) : '')

  // DISPATCH REORDER
  const reorder = ({ from, to }) => {
    let newList = [...players];
    newList[from] = [newList[to], (newList[to] = newList[from])][0];

    onupdate?.(newList);
  }
</script>

<style>
  .entity-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.625em 0.75em;
    border-radius: 8px;
    border: 1px solid var(--color-edge);
    background-color: var(--color-surface);
    transition: border 0.2s linear, opacity 0.3s;
  }
  .entity-row.over { opacity: 0.8; }
  .entity-row.active {
    border-color: var(--color-primary);
    background-color: color-mix(in oklab, var(--color-primary) 10%, var(--color-surface));
  }
  .entity-row.dead .name { text-decoration: line-through; }

  /* Presenter (initiative) list keeps its existing dimmed look. */
  .list-group.initiative-list .entity-row { opacity: 0.8; }

  .entity-main { display: flex; align-items: center; gap: 0.625em; min-width: 0; }
  .drag-handle { color: var(--color-muted); cursor: grab; flex: 0 0 auto; }

  .init-badge {
    background-color: var(--color-elevated);
    min-width: 1.875em;
    height: 1.875em;
    padding: 0 0.4em;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    font-weight: bold;
    flex: 0 0 auto;
  }
  .entity-row.active .init-badge {
    background-color: var(--color-primary);
    color: #000;
  }

  .entity-info { min-width: 0; }
  .entity-info .name { font-weight: 600; font-size: 0.875em; }
  .entity-info .meta { font-size: 0.6875em; color: var(--color-muted); }

  .entity-actions { display: flex; align-items: center; gap: 0.5em; flex: 0 0 auto; }
  .hp-badge {
    background: var(--color-edge);
    padding: 0.25em 0.5em;
    border-radius: 12px;
    font-size: 0.6875em;
    font-family: monospace;
    white-space: nowrap;
  }
</style>

<div class="list-group {initiative ? 'initiative-list' : ''}" >
  {#each players || [] as player, index (player.id)}
    <div
      draggable={!initiative && sortable}
      role="note"
      class="entity-row list-group-item list-group-item-action"
      class:active={player.active}
      class:dead={player.dead}
      class:over={player.id === isOver}
      data-index={index}
      data-id={player.id}
      ondragstart={start}
      ondragover={over}
      ondragleave={leave}
      ondrop={drop}
      in:receive={{ key: player.id }}
      out:send={{ key: player.id }}
      animate:flip={{ duration: 300 }}>
      <div class="entity-main">
        {#if !initiative && sortable}
          <i class="fa-solid fa-grip-vertical drag-handle"></i>
        {/if}
        {#if showInitiativeRoll}
          <div class="init-badge">
            <InPlaceEdit bind:value={player.initiative} onsubmit={updateField(player.id, 'initiative')} editable={!initiative && !sortable} />
          </div>
        {/if}
        <div class="entity-info">
          <div class="name">
            <InPlaceEdit bind:value={player.name} onsubmit={updateField(player.id, 'name')} editable={!initiative && !sortable} />
          </div>
          <div class="meta">{kindLabel(player.kind)}{#if player.dead} • Dead{/if}</div>
        </div>
      </div>
      <div class="entity-actions">
        {#if (healthVisible && enemyHealthVisible) || (healthVisible && (player.kind == 'player' || player.kind == 'npc' )) }
          <span class="hp-badge">
            <InPlaceEdit bind:value={player.health} onsubmit={updateField(player.id, 'health')} editable={!initiative && !sortable} />
            /&nbsp;<InPlaceEdit bind:value={player.maxHealth} onsubmit={updateField(player.id, 'maxHealth')} editable={!initiative && !sortable} />
          </span>
        {:else if healthVisible && !enemyHealthVisible && player.kind == 'monster' }
          <span class="hp-badge">{player.maxHealth - player.health === 0 ? '' : '-'}{player.maxHealth - player.health}</span>
        {/if}
        {#if !initiative && !sortable}
          <button class="btn btn-sm btn-outline-danger" aria-label="delete player" onclick={deletePlayer(player.id)}>
            <i class="fa-solid fa-square-xmark"></i>
          </button>
        {/if}
      </div>
    </div>
  {/each}
</div>
