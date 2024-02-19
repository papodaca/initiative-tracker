import { Store } from "tauri-plugin-store-api"
import { emit } from "@tauri-apps/api/event"

const store = new Store(".settings.dat");

export let getState = () => store.get("state")

export let setState = async (state) => {
    await store.set("state", state)
    emit('state-change', state)
}

export default store
