import { LazyStore } from "@tauri-apps/plugin-store"
import { emit } from "@tauri-apps/api/event"

let store = null

let makeStore = async () => {
  if (store === null) {
    store = await new LazyStore(".settings.dat")
  }
}

export let getState = async () => {
  await makeStore()
  return store.get("state")
}

export let setState = async (state) => {
  await makeStore()
  await store.set("state", state)
  emit('state-change', state)
}

export let saveStore = async () => {
  await makeStore()
  store.save()
}

export default store
