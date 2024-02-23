import { Store } from "@tauri-apps/plugin-store"
import { emit } from "@tauri-apps/api/event"

const store = new Store(".settings.dat");

export const getState = () => store.get("state")

export const setState = async (state) => {
    await store.set("state", state)
    emit('state-change', state)
}

export const saveStore = () => store.save()
