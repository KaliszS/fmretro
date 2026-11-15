<script lang="ts">
  import { fly, fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { dur } from "../motion";

  type Props = {
    kind: "info" | "ok" | "err";
    text: string;
    onDismiss: () => void;
  };

  let { kind, text, onDismiss }: Props = $props();
</script>

<div
  class="toast {kind}"
  role={kind === "err" ? "alert" : "status"}
  in:fly={{ y: 12, duration: dur(260), easing: cubicOut }}
  out:fade={{ duration: dur(160) }}
>
  <span class="icon" aria-hidden="true">
    {#if kind === "ok"}
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
        <path d="M3.5 8.5l3 3 6-7" />
      </svg>
    {:else if kind === "err"}
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
        <path d="M8 4.5v4.25M8 11.25v.25" />
      </svg>
    {:else}
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
        <path d="M8 7.25v4.25M8 4.5v.25" />
      </svg>
    {/if}
  </span>
  <span class="msg">{text}</span>
  <button type="button" class="dismiss" aria-label="Dismiss" onclick={onDismiss}>
    <svg viewBox="0 0 12 12" width="10" height="10" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
      <path d="M3 3l6 6M9 3l-6 6" />
    </svg>
  </button>
</div>

<style>
  .toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    max-width: 100%;
    padding: 10px 10px 10px 12px;
    border-radius: var(--r-md);
    background: var(--bg-raised);
    box-shadow: var(--shadow-pop);
    font-size: 12px;
    color: var(--text);
  }

  .icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: 50%;
    color: var(--text-2);
    background: var(--bg-active);
  }

  .ok .icon {
    color: var(--ok);
    background: var(--ok-soft);
  }

  .err .icon {
    color: var(--err);
    background: var(--err-soft);
  }

  .msg {
    flex: 1;
    min-width: 0;
    padding-top: 1px;
    line-height: 1.4;
    overflow-wrap: anywhere;
    user-select: text;
  }

  .dismiss {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: var(--r-xs);
    background: transparent;
    color: var(--text-3);
    cursor: pointer;
  }

  .dismiss:hover {
    background: var(--bg-hover);
    color: var(--text);
  }
</style>
