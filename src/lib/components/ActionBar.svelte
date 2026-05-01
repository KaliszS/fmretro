<script lang="ts">
  type Props = {
    canApply: boolean;
    canRestore: boolean;
    busy: boolean;
    onApply: () => void;
    onRestore: () => void;
  };

  let { canApply, canRestore, busy, onApply, onRestore }: Props = $props();
</script>

<div class="actions">
  <button
    type="button"
    class="btn primary"
    disabled={!canApply || busy}
    onclick={onApply}
  >
    {#if busy}
      <span class="spinner" aria-hidden="true"></span>
      Working…
    {:else}
      <span class="ico" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.2">
          <path d="M13 2 4 14h7l-1 8 9-12h-7l1-8z" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </span>
      Apply patch
    {/if}
  </button>
  <button
    type="button"
    class="btn ghost"
    disabled={!canRestore || busy}
    onclick={onRestore}
  >
    <span class="ico" aria-hidden="true">
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 12a9 9 0 1 0 3-6.7" stroke-linecap="round" stroke-linejoin="round"/>
        <path d="M3 4v5h5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </span>
    Restore
  </button>
</div>

<style>
  .actions {
    display: flex;
    gap: 10px;
  }

  .btn {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: var(--r-md);
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    border: 1px solid var(--border);
    color: var(--text);
    background: var(--surface-1);
    transition: all 0.15s ease;
  }

  .btn:hover:not(:disabled) {
    border-color: var(--border-strong);
    transform: translateY(-1px);
  }

  .btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn.primary {
    background: var(--accent-grad);
    color: #ffffff;
    border-color: transparent;
    box-shadow: var(--accent-glow);
  }

  .btn.primary:hover:not(:disabled) {
    filter: brightness(1.08);
    box-shadow: 0 12px 36px -8px var(--accent-b-soft);
  }

  .ico {
    display: inline-grid;
    place-items: center;
  }

  .spinner {
    width: 13px;
    height: 13px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
