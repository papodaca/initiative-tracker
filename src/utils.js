
export const toTitleCase = (str) => {
  return str.replace(
    /\w\S*/g,
    function(txt) {
      return txt.charAt(0).toUpperCase() + txt.substr(1).toLowerCase();
    }
  );
}

export const splitWords = (strings, ...values) =>
  strings.reduce((acc, s, i) => acc + s + (i < values.length ? String(values[i]) : ""), "")
    .trim().split(/\s+/).filter(Boolean)

export const splitNumbers = (strings, ...values) =>
  splitWords(strings, ...values).map(Number)
