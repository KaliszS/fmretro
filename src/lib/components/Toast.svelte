<script lang="ts">
  type Props = {
    kind: "info" | "ok" | "err";
    text: string;
  };
  let { kind, text }: Props = $props();
</script>

{#if text}
  <div class="toast {kind}" role="status">
    <span class="dot"></span>
    <span class="msg">{text}</span>
  </div>
{/if}

<style>
  .toast {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-1);
    font-size: 11px;
    color: var(--text-soft);
    backdrop-filter: blur(10px);
    animation: rise 0.25s ease-out;
  }

  .toast.ok {
    border-color: var(--ok-border);
    color: var(--ok);
    background: var(--ok-bg);
  }

  .toast.err {
    border-color: var(--err-border);
    color: var(--err);
    background: var(--err-bg);
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
    flex-shrink: 0;
  }

  .msg {
    user-select: text;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
