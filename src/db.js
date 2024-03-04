import Dixie from 'dexie'

import { callback, w } from "./utils"

export const db = new Dixie('initiative-tracker')

const models = w`monsters spells classes races weapons armor magicitems`

const dbConfig = {
  [models[0]]: "&slug"
}
models.forEach(model => dbConfig[model] = "&slug");
db.version(1).stores(dbConfig)

export const loadDB = async (...args) => {
  for(let model of models) {
    await loadModel(model, ...args)
  }
}

const loadModel = async(model, force = false, progress = null) => {
  callback('progress', progress, model, 0)
  let morePages = true, nextPage = null, total = 0, fetched = 0
  let count = await db[model].count()
  if (!force && count > 0) return
  while (morePages) {
    let res = await fetch(nextPage || `https://api.open5e.com/${model}/?limit=100`)
    let data = await res.json()
    nextPage = data.next
    morePages = nextPage != null
    for (let datum of data.results) {
      await db[model].put({_model: model, ...datum})
    }
    fetched += data.results.length
    if (total === 0) total = data.count
    callback('progress', progress, model, Math.ceil(fetched / total * 100))
  }
}

export const getModels = async (model) => {
  return await db[model].toArray()
}

export const getModel = async (model, slug) => {
  return await db[model].where("slug").equalsIgnoreCase(slug).first()
}

export default db
