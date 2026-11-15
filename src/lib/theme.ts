// Theme management with localStorage persistence + DOM data attribute.
// The initial value is applied by an inline script in app.html before first paint.
import { writable } from "svelte/store";

export type ThemeMode = "dark" | "light";

const KEY = "fmretro:theme";

function initial(): ThemeMode {
  if (typeof document === "undefined") return "dark";
  const attr = document.documentElement.getAttribute("data-theme");
  if (attr === "dark" || attr === "light") return attr;
  try {
    const saved = window.localStorage.getItem(KEY);
    if (saved === "dark" || saved === "light") return saved;
  } catch {
    // Storage may be unavailable; fall through to system preference.
  }
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

export const theme = writable<ThemeMode>(initial());

theme.subscribe((mode) => {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-theme", mode);
  try {
    window.localStorage.setItem(KEY, mode);
  } catch {
    // Ignore — theme still applies for this session.
  }
});

export function toggleTheme(): void {
  theme.update((m) => (m === "dark" ? "light" : "dark"));
}
