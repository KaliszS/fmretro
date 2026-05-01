<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

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

<div class="titlebar" data-tauri-drag-region>
  <span class="t-title" data-tauri-drag-region>FM Retro Patcher</span>
  <div class="t-controls">
    <button
      type="button"
      class="t-btn"
      title="Minimize"
      aria-label="Minimize"
      onclick={minimize}
    >
      <svg viewBox="0 0 12 12" width="10" height="10">
        <line x1="2" y1="6" x2="10" y2="6" stroke="currentColor" stroke-width="1.4" />
      </svg>
    </button>
    <button
      type="button"
      class="t-btn close"
      title="Close (auto-restores patches)"
      aria-label="Close"
      onclick={close}
    >
      <svg viewBox="0 0 12 12" width="10" height="10">
        <path d="M2 2 L10 10 M10 2 L2 10" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .titlebar {
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 0 14px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-1);
    user-select: none;
    border-top-left-radius: var(--r-lg);
    border-top-right-radius: var(--r-lg);
  }

  .t-title {
    font-size: 11px;
    color: var(--text-muted);
    letter-spacing: 0.5px;
    font-weight: 600;
    pointer-events: none;
  }

  .t-controls {
    display: flex;
    gap: 4px;
  }

  .t-btn {
    width: 22px;
    height: 22px;
    display: grid;
    place-items: center;
    border: none;
    background: transparent;
    color: var(--text-muted);
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  .t-btn:hover {
    background: var(--surface-2);
    color: var(--text);
  }

  .t-btn.close:hover {
    background: var(--err-bg);
    color: var(--err);
  }
</style>
