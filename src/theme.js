// Pure theme helpers. Preference ("light" | "dark" | "system") is owned by the
// Console component and persisted on the existing Tauri store state as `state.theme`.
// The Presenter applies the effective theme on each `state-change` event.

export const getSystemTheme = () => {
  if (typeof window === "undefined" || !window.matchMedia) return "dark"
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
}

export const getEffectiveTheme = (preference) =>
  preference === "light" || preference === "dark" ? preference : getSystemTheme()

export const applyTheme = (preference) => {
  if (typeof document === "undefined") return
  document.documentElement.dataset.theme = getEffectiveTheme(preference)
}

// Subscribe to OS preference changes. Returns an unsubscribe function.
export const watchSystemTheme = (callback) => {
  if (typeof window === "undefined" || !window.matchMedia) return () => {}
  const mq = window.matchMedia("(prefers-color-scheme: dark)")
  const handler = () => callback(getSystemTheme())
  mq.addEventListener("change", handler)
  return () => mq.removeEventListener("change", handler)
}
