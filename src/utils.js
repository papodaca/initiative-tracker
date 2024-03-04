
export const toTitleCase = (str) => {
  return str.replace(
    /\w\S*/g,
    function(txt) {
      return txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase();
    }
  );
}

export const w = (strings, ...args) => {
  if (strings.length == 1) return strings[0].split(' ')
  return strings.map((string, idx) => `${string}${args[idx] ? args[idx] : ''}`).join('').split(' ')
}

export const s = (...args) => w(...args).map(Symbol)
export const i = (...args) => w(...args).map(parseInt)
export const f = (...args) => w(...args).map(parseFloat)

export const callback = async (name, cb, ...args) => {
  try {
    let res = cb?.(...args)
    if (res instanceof Promise) await res
  } catch (e) {
    console.warn(`error in ${name}`, e)
  }
}