<script lang="ts">
  const version: string = __APP_VERSION__;
</script>

<span class="badge" title="App version">v{version}</span>

<style>
  .badge {
    position: fixed;
    right: 10px;
    bottom: 8px;
    z-index: 5;
    padding: 3px 8px;
    border-radius: 999px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.4px;
    color: var(--text-muted);
    background: var(--surface-1);
    border: 1px solid var(--border);
    backdrop-filter: blur(8px);
    user-select: none;
    pointer-events: auto;
    transition: color 0.2s, border-color 0.2s;
    font-variant-numeric: tabular-nums;
  }

  .badge:hover {
    color: var(--text-soft);
    border-color: var(--border-strong);
  }
</style>
