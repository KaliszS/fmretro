<script lang="ts">
  import { flap } from "../motion";

  type Props = {
    /** Season as the game shows it, e.g. "2023/24". Null when unknown. */
    source: string | null;
    /** Season after patching, e.g. "1986/87". Null when not ready. */
    target: string | null;
    /** Years shifted back, shown as a small caption. */
    shift: number | null;
  };

  let { source, target, shift }: Props = $props();

  const EMPTY = "    /  ";

  function cells(season: string | null): string[] {
    return (season && season.length === 7 ? season : EMPTY).split("");
  }

  let sourceCells = $derived(cells(source));
  let targetCells = $derived(cells(target));
</script>

{#snippet tiles(list: string[])}
  {#each list as ch, i (i)}
    {#if ch === "/"}
      <span class="slash">/</span>
    {:else}
      <span class="tile" class:blank={ch === " "}>
        {#key ch}<span class="glyph" in:flap={{ delay: i * 28 }}>{ch === " " ? "" : ch}</span>{/key}
      </span>
    {/if}
  {/each}
{/snippet}

<div class="board" role="img" aria-label={source && target ? `${source} becomes ${target}` : "Season preview"}>
  <div class="row">
    <span class="caption">In game</span>
    <div class="tiles">{@render tiles(sourceCells)}</div>
  </div>

  <div class="row">
    <span class="caption">You'll see</span>
    <div class="tiles target">{@render tiles(targetCells)}</div>
  </div>

  <div class="foot">
    {#if shift !== null && target}
      <span class="shift mono">−{shift} years</span>
    {:else}
      <span class="shift idle">Enter a year to preview</span>
    {/if}
  </div>
</div>

<style>
  .board {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 16px 12px;
    border-radius: var(--r-lg);
    background: var(--bg-sunken);
    border: 1px solid var(--line);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .caption {
    font-size: 12px;
    color: var(--text-2);
    white-space: nowrap;
  }

  .tiles {
    display: flex;
    align-items: center;
    gap: 3px;
  }

  .tile {
    position: relative;
    width: 28px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: var(--r-xs);
    background: var(--tile-bg);
    color: var(--tile-text);
    font-family: var(--font-mono);
    font-size: 22px;
    font-weight: 600;
    line-height: 1;
    box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.35);
    overflow: hidden;
  }

  /* Hinge line of the split-flap. */
  .tile::after {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: 50%;
    height: 1px;
    background: var(--tile-line);
  }

  .tile.blank {
    opacity: 0.55;
  }

  .glyph {
    display: block;
    transform-origin: 50% 50%;
    backface-visibility: hidden;
  }

  .target .tile {
    color: var(--accent);
  }

  .slash {
    width: 12px;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 18px;
    color: var(--text-3);
  }

  .foot {
    display: flex;
    justify-content: flex-end;
    min-height: 16px;
  }

  .shift {
    font-size: 11px;
    color: var(--accent-text);
  }

  .shift.idle {
    font-family: var(--font-sans);
    color: var(--text-3);
  }
</style>
