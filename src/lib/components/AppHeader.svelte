<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import ThemeToggle from "./ThemeToggle.svelte";

  type Props = {
    debugActive: boolean;
    onToggleDebug: () => void;
  };

  let { debugActive, onToggleDebug }: Props = $props();

  async function minimize() {
    try {
      await getCurrentWindow().minimize();
    } catch (e) {
      console.warn(e);
    }
  }

  async function close() {
    try {
      await getCurrentWindow().close();
    } catch (e) {
      console.warn(e);
    }
  }
</script>

<header class="bar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <span class="mark" aria-hidden="true">
      <!-- Rewind glyph: the app turns the calendar back. -->
      <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
        <path d="M7.5 3.6v8.8a.5.5 0 0 1-.8.4L1.4 8.4a.5.5 0 0 1 0-.8l5.3-4.4a.5.5 0 0 1 .8.4z" />
        <path d="M14.5 3.6v8.8a.5.5 0 0 1-.8.4L8.4 8.4a.5.5 0 0 1 0-.8l5.3-4.4a.5.5 0 0 1 .8.4z" />
      </svg>
    </span>
    <span class="name" data-tauri-drag-region>FM Retro</span>
  </div>

  <div class="tools">
    <button
      type="button"
      class="icon-btn"
      title={debugActive ? "Back to patcher" : "Diagnostics"}
      aria-label="Toggle diagnostics"
      aria-pressed={debugActive}
      onclick={onToggleDebug}
    >
      <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
        <path d="M2 4.5h7M12 4.5h2M2 11.5h2M7 11.5h7" />
        <circle cx="10.5" cy="4.5" r="1.5" />
        <circle cx="5.5" cy="11.5" r="1.5" />
      </svg>
    </button>
    <ThemeToggle />

    <span class="sep" aria-hidden="true"></span>

    <button type="button" class="icon-btn win" title="Minimize" aria-label="Minimize" onclick={minimize}>
      <svg viewBox="0 0 12 12" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.1">
        <path d="M2 6h8" />
      </svg>
    </button>
    <button
      type="button"
      class="icon-btn win close"
      title="Close — restores the game automatically"
      aria-label="Close"
      onclick={close}
    >
      <svg viewBox="0 0 12 12" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.1" stroke-linecap="round">
        <path d="M2.5 2.5l7 7M9.5 2.5l-7 7" />
      </svg>
    </button>
  </div>
</header>

<style>
  .bar {
    height: 44px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 0 16px;
    border-bottom: 1px solid var(--line);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    min-width: 0;
  }

  .mark {
    width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
    border-radius: 6px;
    background: var(--tile-bg);
    color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--line-strong);
    pointer-events: none;
  }

  .mark svg {
    display: block;
    margin-left: -1px;
  }

  .name {
    font-family: var(--font-display);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.005em;
    color: var(--text);
  }

  .tools {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .sep {
    width: 1px;
    height: 16px;
    margin: 0 6px;
    background: var(--line-strong);
  }

  .win {
    color: var(--text-3);
  }

  .close:hover {
    background: #e5484d;
    color: #fff;
  }
</style>
