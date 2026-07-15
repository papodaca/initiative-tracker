<script>
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { open } from '@tauri-apps/plugin-dialog'
  import { splitWords as w } from '../utils'
  import InPlaceEdit from "./InPlaceEdit.svelte"

  let { images = [], onupdate } = $props()

  const updateField = (index, field) => ((_e) => {
    onupdate?.(images);
  })

  const openFiles = () => (open({
    multiple: true,
    directory: false,
    filters: [{
      name: "Image",
      extensions: w`avif ico jfif svg png jpeg jpg webp bmp gif`
    }]
  }))

  const openImages = async () => {
    let files = await openFiles()
    if (files == null) return

    const newImages = [...images]
    for (let filePath of files) {
      let name = filePath.split("/").pop().replace(/\.[^.]+$/, "")
      newImages.push({
        id: crypto.randomUUID(),
        name,
        fileUrl: convertFileSrc(filePath),
        active: false
      })
    }
    onupdate?.(newImages);
  }
  const makeActive = (index) => ((_e) => {
    const newImages = images.map((i, idx) => ({ ...i, active: idx === index }))
    onupdate?.(newImages);
  })
  const keydown = (_e) => {}
</script>

<style>
  .list-group {
    max-width: 600px;
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
      <div class="image" style="--bg-image: url({image.fileUrl})" onclick={makeActive(index)} role="button" aria-label="make {image.name} active" tabindex={index} onkeydown={keydown}></div>
      <div class="name"><InPlaceEdit bind:value={image.name} onsubmit={updateField(index, 'name')} editable={true} /></div>
    </div>
  {/each}
</div>
<button class="btn btn-success" onclick={openImages}>
  <i class="fa-regular fa-square-plus"></i>&nbsp;Add Images
</button>
