<script lang="ts">
  import { theme, toggleTheme } from "../theme";

  let mode = $state<"dark" | "light">("dark");
  theme.subscribe((m) => (mode = m));
</script>

<button
  type="button"
  class="icon-btn theme-btn"
  title={mode === "dark" ? "Switch to light theme" : "Switch to dark theme"}
  aria-label="Toggle theme"
  onclick={toggleTheme}
>
  {#if mode === "dark"}
    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M21 12.79A9 9 0 1 1 11.21 3a7 7 0 0 0 9.79 9.79z" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
  {:else}
    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" stroke-linecap="round"/>
    </svg>
  {/if}
</button>

<style>
  .theme-btn:hover {
    transform: rotate(15deg);
  }
</style>
