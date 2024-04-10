<script>
  import { createEventDispatcher } from "svelte"
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { open } from '@tauri-apps/plugin-dialog'
  import InPlaceEdit from "./InPlaceEdit.svelte"
  const dispatch = createEventDispatcher();

  export let images = []

  const updateField = (index, field) => ((_e) => {
    dispatch("update", images);
  })

  const openFiles = () => (open({
    multiple: true,
    directory: false,
    filters: [{
      name: "Image",
      extensions: ["png", "jpeg", "jpg", "webp", "bmp", "tiff", "gif"]
    }]
  }))

  const openImages = async () => {
    let files = await openFiles() 
    if (files == null) return

    for(let file of files) {
      let filenameExt = file.name.split(".")
      let name = filenameExt.slice(0, filenameExt.length - 1).join(".")
      images.push({
        id: crypto.randomUUID(),
        name,
        fileUrl: convertFileSrc(file.path),
        active: false
      })
    }
    dispatch("update", images);
  }
  const makeActive = (index) => ((_e) => {
    images.forEach(i => i.active = false)
    images[index].active = true
    dispatch("update", images);
  })
  const keydown = (_e) => {}
</script>

<style>
  .list-group {
    max-width: 600px;
    max-height: 1000px;
    overflow: scroll;
  }
  .list-group-item .image {
    display: inline-block;
    background: rgba(0, 0, 0, 0) var(--bg-image);
    background-size: contain;
    background-repeat: no-repeat;
    width: 120px;
    height: 90px;
  }
  .list-group-item .name {
    display: inline-block;
    margin-left: 0.5em;
  }
</style>

<div class="list-group">
  {#each images || [] as image, index (image.id)}
    <div
      class="list-group-item"
      class:active={image.active}>
      <div class="image" style="--bg-image: url({image.fileUrl})" on:click={makeActive(index)} role="button" aria-label="make {image.name} active" tabindex={index} on:keydown={keydown}/>
      <div class="name"><InPlaceEdit bind:value={image.name} on:submit={updateField(index, 'name')} editable={true} /></div>
    </div>
  {/each}
</div>
<button class="btn btn-success" on:click={openImages}>
  <i class="fa-regular fa-square-plus"></i>&nbsp;Add Images
</button>
