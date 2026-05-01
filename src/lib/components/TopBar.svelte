<script lang="ts">
  import ConnectionPill from "./ConnectionPill.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";

  type Props = {
    connected: boolean;
    debugActive: boolean;
    onToggleDebug: () => void;
  };

  let { connected, debugActive, onToggleDebug }: Props = $props();
</script>

<header class="topbar">
  <div class="brand">
    <div class="logo" aria-hidden="true">
      <span class="ring"></span>
      <span class="core">FM</span>
    </div>
    <div class="title">
      <h1>FM Retro Patcher</h1>
      <p>Travel through time, one season at a time.</p>
    </div>
  </div>
  <div class="right">
    <ConnectionPill {connected} />
    <button
      type="button"
      class="icon-btn"
      class:active={debugActive}
      title={debugActive ? "Hide debug info" : "Show debug info"}
      aria-label="Toggle debug panel"
      aria-pressed={debugActive}
      onclick={onToggleDebug}
    >
      <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="9" y="6" width="6" height="12" rx="3" />
        <path d="M9 9 5 7M9 12H4M9 15l-4 2M15 9l4-2M15 12h5M15 15l4 2M12 6V3" stroke-linecap="round"/>
      </svg>
    </button>
    <ThemeToggle />
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 2px 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .logo {
    position: relative;
    width: 38px;
    height: 38px;
    display: grid;
    place-items: center;
    border-radius: 12px;
    background:
      linear-gradient(135deg, var(--accent-a-soft), var(--accent-b-soft)),
      var(--surface-2);
    border: 1px solid var(--accent-a-border);
    box-shadow: var(--accent-glow);
  }

  .core {
    font-weight: 800;
    font-size: 13px;
    letter-spacing: 0.5px;
    background: var(--accent-grad);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }

  .ring {
    position: absolute;
    inset: -2px;
    border-radius: 14px;
    background: conic-gradient(
      from 0deg,
      transparent 0%,
      var(--accent-b) 30%,
      transparent 60%
    );
    filter: blur(4px);
    opacity: 0.45;
    animation: spin 6s linear infinite;
    pointer-events: none;
  }

  .title h1 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
    letter-spacing: 0.2px;
  }

  .title p {
    margin: 1px 0 0;
    font-size: 11px;
    color: var(--text-muted);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
