import Database from "@tauri-apps/plugin-sql"
import { get } from "svelte/store";

const dbName = "sqlite:compendium.db"

let db;

const getDB = async () => {
    if (db == null) db = await Database.load(dbName)
    return db
}

const insertMonsters = async (monsters) => {
    for (let monster of monsters) {
        await db.execute(
            "INSERT INTO monsters VALUES ($1, $2);",
            [monster.slug, monster]
        )
    }
}

export const loadDB = async (force = false, progress = (progress) => {}) => {
    await getDB();
    let morePages = true, nextPage = null, total = 0, fetched = 0
    let res = await db.select("SELECT COUNT(*) as count FROM monsters;")
    if (!force && res[0].count > 0) return
    await db.execute("DELETE from monsters;")
    while(morePages) {
        let res = await fetch(nextPage || "https://api.open5e.com/monsters/?limit=100")
        let data = await res.json()
        nextPage = data.next
        morePages = nextPage != null
        await insertMonsters(data.results)
        fetched += data.results.length
        if (total === 0) total = data.count
        try {
          progress(Math.ceil(fetched / total * 100))
        } catch (e) {
            console.warn("error in progress", e)
        }
    }
}

export const getMonsters = async () => {
    await getDB();
    return await db.select("SELECT * FROM monsters ORDER BY slug DESC;")
}

export const getMonster = async (slug) => {
    await getDB();
    return await db.select("SELECT * FROM monsters where slug = $1", [slug])
}

export const closeDB = async () => {
    await getDB()
    await db.close(dbName)
}

export default getDB
