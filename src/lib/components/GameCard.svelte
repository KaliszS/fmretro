<script lang="ts">
  import type { FmEdition } from "../types";
  import { formatSeason } from "../editions";

  type Props = {
    connected: boolean;
    edition: FmEdition | null;
    pid: number | null;
    patchActive: boolean;
  };

  let { connected, edition, pid, patchActive }: Props = $props();

  let phase = $derived<"idle" | "reading" | "ready" | "patched">(
    !connected ? "idle" : !edition ? "reading" : patchActive ? "patched" : "ready",
  );
</script>

<section class="game" data-state={phase} aria-live="polite">
  <span class="indicator" aria-hidden="true"></span>

  <div class="text">
    {#if phase === "idle"}
      <span class="title">Waiting for Football Manager</span>
      <span class="sub">Start the game — it will be picked up automatically.</span>
    {:else if phase === "reading" || !edition}
      <span class="title">Game found</span>
      <span class="sub">Reading game version…</span>
    {:else}
      <span class="title">Football Manager {edition.startYear + 1}</span>
      <span class="sub">
        Starts in {formatSeason(edition.startYear)}
        {#if pid !== null}<span class="dot">·</span><span class="mono">PID {pid}</span>{/if}
      </span>
    {/if}
  </div>

  {#if phase === "ready"}
    <span class="chip ok">Connected</span>
  {:else if phase === "patched"}
    <span class="chip accent">Patch active</span>
  {/if}
</section>

<style>
  .game {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border: 1px solid var(--line);
    border-radius: var(--r-lg);
    background: var(--bg-raised);
    transition: border-color 200ms ease;
  }

  .game[data-state="idle"] {
    background: transparent;
    border-style: dashed;
    border-color: var(--line-strong);
  }

  .indicator {
    position: relative;
    width: 8px;
    height: 8px;
    flex-shrink: 0;
    border-radius: 50%;
    background: var(--text-3);
    transition: background-color 200ms ease;
  }

  [data-state="idle"] .indicator::after,
  [data-state="reading"] .indicator::after {
    content: "";
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    border: 1px solid currentColor;
    color: var(--text-3);
    animation: ping 1.8s var(--ease-out) infinite;
  }

  [data-state="ready"] .indicator {
    background: var(--ok);
  }

  [data-state="patched"] .indicator {
    background: var(--accent);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .title {
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .sub {
    font-size: 12px;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sub .mono {
    font-size: 11px;
    color: var(--text-3);
  }

  .dot {
    margin: 0 6px;
    color: var(--text-3);
  }

  .chip {
    flex-shrink: 0;
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 999px;
    animation: chip-in 240ms var(--ease-out);
  }

  .chip.ok {
    color: var(--ok);
    background: var(--ok-soft);
  }

  .chip.accent {
    color: var(--accent-text);
    background: var(--accent-soft);
  }

  @keyframes ping {
    0% {
      transform: scale(0.5);
      opacity: 0.9;
    }
    100% {
      transform: scale(1.6);
      opacity: 0;
    }
  }

  @keyframes chip-in {
    from {
      opacity: 0;
      transform: scale(0.92);
    }
  }
</style>
