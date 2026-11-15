<script lang="ts">
  import { theme, toggleTheme } from "../theme";
</script>

<button
  type="button"
  class="icon-btn"
  title={$theme === "dark" ? "Light theme" : "Dark theme"}
  aria-label="Toggle theme"
  onclick={toggleTheme}
>
  <span class="swap" class:light={$theme === "light"}>
    <svg class="moon" viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
      <path d="M13.5 9.6A5.75 5.75 0 0 1 6.4 2.5a5.75 5.75 0 1 0 7.1 7.1z" />
    </svg>
    <svg class="sun" viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
      <circle cx="8" cy="8" r="2.75" />
      <path d="M8 1.5v1.25M8 13.25v1.25M1.5 8h1.25M13.25 8h1.25M3.4 3.4l.9.9M11.7 11.7l.9.9M3.4 12.6l.9-.9M11.7 4.3l.9-.9" />
    </svg>
  </span>
</button>

<style>
  .swap {
    position: relative;
    width: 15px;
    height: 15px;
    display: block;
  }

  .swap svg {
    position: absolute;
    inset: 0;
    transition:
      opacity 180ms ease,
      transform 260ms var(--ease-out);
  }

  .sun {
    opacity: 0;
    transform: rotate(-45deg) scale(0.6);
  }

  .light .moon {
    opacity: 0;
    transform: rotate(45deg) scale(0.6);
  }

  .light .sun {
    opacity: 1;
    transform: none;
  }
</style>
