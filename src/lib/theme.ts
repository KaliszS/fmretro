// Theme management with localStorage persistence + DOM data attribute.
import { writable } from "svelte/store";

export type ThemeMode = "dark" | "light";

const KEY = "fmretro:theme";

function initial(): ThemeMode {
  if (typeof window === "undefined") return "dark";
  const saved = window.localStorage.getItem(KEY);
  if (saved === "dark" || saved === "light") return saved;
  const prefersLight = window.matchMedia("(prefers-color-scheme: light)").matches;
  return prefersLight ? "light" : "dark";
}

function apply(mode: ThemeMode): void {
  if (typeof document === "undefined") return;
  document.documentElement.setAttribute("data-theme", mode);
}

export const theme = writable<ThemeMode>(initial());

theme.subscribe((mode) => {
  if (typeof window === "undefined") return;
  apply(mode);
  window.localStorage.setItem(KEY, mode);
});

export function toggleTheme(): void {
  theme.update((m) => (m === "dark" ? "light" : "dark"));
}
