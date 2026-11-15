<script lang="ts">
  type Props = {
    patchActive: boolean;
    canApply: boolean;
    canRestore: boolean;
    busy: boolean;
    onApply: () => void;
    onRestore: () => void;
  };

  let { patchActive, canApply, canRestore, busy, onApply, onRestore }: Props = $props();

  let disabled = $derived(busy || (patchActive ? !canRestore : !canApply));
</script>

<button
  type="button"
  class="btn"
  class:primary={!patchActive}
  class:secondary={patchActive}
  class:busy
  {disabled}
  aria-busy={busy}
  onclick={patchActive ? onRestore : onApply}
>
  <span class="content">
    {#if patchActive}
      <svg viewBox="0 0 16 16" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M2.75 8a5.25 5.25 0 1 0 1.6-3.78" />
        <path d="M2.5 2.5v2.75h2.75" />
      </svg>
      Restore original years
    {:else}
      Apply patch
    {/if}
  </span>
  {#if busy}
    <span class="spinner" aria-hidden="true"></span>
  {/if}
</button>

<style>
  .btn {
    position: relative;
    width: 100%;
    height: 42px;
    display: grid;
    place-items: center;
    padding: 0 16px;
    border-radius: var(--r-md);
    border: 1px solid transparent;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background-color 140ms ease,
      border-color 140ms ease,
      color 140ms ease,
      opacity 140ms ease,
      transform 100ms ease;
  }

  .btn:active:not(:disabled) {
    transform: scale(0.985);
  }

  .content {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    transition: opacity 140ms ease;
  }

  .primary {
    background: var(--accent);
    color: var(--accent-ink);
  }

  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .secondary {
    background: var(--bg-raised);
    border-color: var(--line-strong);
    color: var(--text);
  }

  .secondary:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn:disabled {
    cursor: not-allowed;
  }

  .primary:disabled {
    background: var(--bg-active);
    color: var(--text-3);
  }

  .secondary:disabled {
    color: var(--text-3);
  }

  /* While busy keep the button's colors; just swap the label for a spinner. */
  .btn.busy {
    cursor: progress;
  }

  .primary.busy:disabled {
    background: var(--accent);
    color: var(--accent-ink);
  }

  .busy .content {
    opacity: 0;
  }

  .spinner {
    position: absolute;
    inset: 0;
    margin: auto;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid currentColor;
    border-right-color: transparent;
    animation: spin 700ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
