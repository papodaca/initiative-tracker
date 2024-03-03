import Dixie from 'dexie'

export const db = new Dixie('initiative-tracker')
db.version(1).stores({
    monsters: "&slug"
})

const insertMonsters = async (monsters) => {
    for (let monster of monsters) {
        await db.monsters.add(monster)
    }
}

export const loadDB = async (force = false, progress = (progress) => {}) => {
    let morePages = true, nextPage = null, total = 0, fetched = 0
    let count = await db.monsters.count()
    if (!force && count > 0) return
    if (force) await db.monsters.delete()
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
    return await db.monsters.toArray()
}

export const getMonster = async (slug) => {
    return await db.monsters.where("slug").equalsIgnoreCase(slug).first()
}

export default db
