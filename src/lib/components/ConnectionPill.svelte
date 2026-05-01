<script lang="ts">
  let { connected }: { connected: boolean } = $props();
</script>

<div class="pill" class:on={connected}>
  <span class="led"></span>
  <span class="text">
    {connected ? "fm.exe connected" : "Waiting for fm.exe…"}
  </span>
</div>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 9px;
    padding: 6px 12px 6px 10px;
    border-radius: 999px;
    background: var(--surface-1);
    border: 1px solid var(--border);
    font-size: 11px;
    color: var(--text-muted);
    backdrop-filter: blur(8px);
    transition: color 0.2s, border-color 0.2s, background 0.2s;
  }

  .pill.on {
    color: var(--ok);
    border-color: var(--ok-border);
    background: var(--ok-bg);
  }

  .led {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-muted);
    flex-shrink: 0;
    transition: background 0.2s, box-shadow 0.2s;
  }

  .pill.on .led {
    background: var(--ok);
    animation: pulse 1.8s ease-out infinite;
  }

  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 var(--ok-pulse);
    }
    70% {
      box-shadow: 0 0 0 7px transparent;
    }
    100% {
      box-shadow: 0 0 0 0 transparent;
    }
  }
</style>
