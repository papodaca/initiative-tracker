<script>
  // based on https://github.com/brunomolteni/svelte-sortable-list/
  import { quintOut } from "svelte/easing";
  import { crossfade } from "svelte/transition";
  import { flip } from "svelte/animate";
  import InPlaceEdit from "./InPlaceEdit.svelte";

  // PROPS
  let { players = [], initiative = true, sortable = false, healthVisible = false, enemyHealthVisible = false, onupdate } = $props()

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
  const updateField = (id, field) => {
    return (event) => {
      onupdate?.(players);
    }
  }
  const deletePlayer = (id) => ((_event) => {
    const newList = players.filter(p => p.id !== id)
    onupdate?.(newList)
  })

  // DISPATCH REORDER
  const reorder = ({ from, to }) => {
    let newList = [...players];
    newList[from] = [newList[to], (newList[to] = newList[from])][0];

    onupdate?.(newList);
  }
</script>

<style>
  .list-group-item {
    transition: border 0.2s linear;
  }
  .over {
    opacity: 0.8;
  }
  .list-group.initiative-list .list-group-item {
    opacity: 0.8;
    transition: opacity 0.3s;
  }
  .list-group.initiative-list .list-group-item div.text.dead .name {
    text-decoration: line-through;
  }

  .list-group.initiative-list .list-group-item div.text {
    opacity: 0.9;
    background-color: var(--bs-body-bg);
  }
  .list-group.initiative-list .list-group-item.active div.text {
    opacity: 0.9;
    background-color: var(--bs-active-bg);
  }

  .list-group .list-group-item div.text .initiative {
    width: 1.5em;
    display: inline-block;
  }
  .list-group .list-group-item div.text .name {
    margin-left: 0.5em;
    margin-right: 0.5em;
  }
  .text .right {
    float: right;
  }
</style>

<div class="list-group {initiative ? 'initiative-list' : ''}" >
  {#each players || [] as player, index (player.id)}
    <div
      draggable={!initiative && sortable}
      role="note"
      class="list-group-item list-group-item-action"
      class:active={player.active}
      data-index={index}
      data-id={player.id}
      ondragstart={start}
      ondragover={over}
      ondragleave={leave}
      ondrop={drop}
      in:receive={{ key: player.id }}
      out:send={{ key: player.id }}
      animate:flip={{ duration: 300 }}
      class:over={player.id === isOver}>
      <div class="text" class:text-danger={player.dead} class:dead={player.dead}>
        {#if !initiative && sortable}
          <i class="fa-solid fa-grip-vertical"></i>&nbsp;
        {/if}
        <span class="initiative"><InPlaceEdit bind:value={player.initiative} onsubmit={updateField(player.id, 'initiative')} editable={!initiative && !sortable} /></span>
        {#if (healthVisible && enemyHealthVisible) || (healthVisible && (player.kind == 'player' || player.kind == 'npc' )) }
        <span class="health">
          HP:&nbsp;<InPlaceEdit bind:value={player.health} onsubmit={updateField(player.id, 'health')} editable={!initiative && !sortable} />
          &nbsp;/&nbsp;
          <InPlaceEdit bind:value={player.maxHealth} onsubmit={updateField(player.id, 'maxHealth')} editable={!initiative && !sortable} />
        </span>
        {:else if healthVisible && !enemyHealthVisible && player.kind == 'monster' }
          HP: ({player.maxHealth - player.health === 0 ? '' : '-'}{player.maxHealth - player.health})
        {/if}
        &nbsp;-&nbsp;
        {#if player.kind === 'player'}
          <i class="fa-solid fa-person-hiking"></i>
        {:else if player.kind === 'npc'}
          <i class="fa-solid fa-user"></i>
        {:else if player.kind ==='monster'}
          <i class="fa-solid fa-dragon"></i>
        {/if}&nbsp;
        <span class="name"><InPlaceEdit bind:value={player.name} onsubmit={updateField(player.id, 'name')} editable={!initiative && !sortable} /></span>
        {#if !initiative && !sortable}
        <div class="right">
          <select bind:value={player.kind} onchange={updateField(player.id, 'kind')} >
            <option value="player">Player</option>
            <option value="npc">NPC</option>
            <option value="monster">Monster</option>
          </select>
          &nbsp;
          <i class="fa-solid fa-skull text-danger"></i>&nbsp;
          <input type="checkbox" bind:checked={player.dead} onchange={updateField(player.id, 'dead')} />&nbsp;
          <button class="btn btn-sm btn-outline-danger" aria-label="delete player" onclick={deletePlayer(player.id)}><i class="fa-solid fa-square-xmark"></i></button>
        </div>
        {/if}
      </div>
    </div>
  {/each}
</div>
