<script lang="ts">
  import NumberField from "./NumberField.svelte";
  import PreviewCard from "./PreviewCard.svelte";
  import ActionBar from "./ActionBar.svelte";
  import { findEdition, formatSeason, editionIdForMajor } from "../editions";
  import type { FmEditionInfo } from "../types";

  type Props = {
    connected: boolean;
    patchActive: boolean;
    busy: boolean;
    detectedEdition: FmEditionInfo | null;
    modStartYear: number | null;
    onApply: (shiftBack: number, minYear: number) => void;
    onRestore: () => void;
  };

  let {
    connected,
    patchActive,
    busy,
    detectedEdition,
    modStartYear = $bindable(),
    onApply,
    onRestore,
  }: Props = $props();

  let detectedId = $derived(
    detectedEdition ? editionIdForMajor(detectedEdition.major) : null,
  );

  let edition = $derived(detectedId ? findEdition(detectedId) : null);

  let inputsLocked = $derived(patchActive || busy || !edition);

  let shiftBack = $derived.by<number | null>(() => {
    if (!edition || modStartYear === null) return null;
    return edition.startYear - modStartYear;
  });

  let modStartYearError = $derived.by<string | null>(() => {
    if (modStartYear === null) return null;
    if (!Number.isInteger(modStartYear)) return "Whole years only.";
    if (modStartYear < 1900 || modStartYear > 2050)
      return "Enter a 4-digit year (e.g. 1986).";
    if (edition && modStartYear >= edition.startYear)
      return `Must be earlier than FM start year (${edition.startYear}).`;
    if (edition && edition.startYear - modStartYear > 100)
      return "Too far back — maximum is 100 years.";
    return null;
  });

  let shiftHint = $derived.by<string>(() => {
    if (shiftBack !== null && modStartYearError === null)
      return `Shift: ${shiftBack} years back.`;
    return "First year of your mod's own historical data.";
  });

  let ready = $derived(
    modStartYear !== null &&
      modStartYearError === null &&
      shiftBack !== null &&
      shiftBack > 0,
  );

  let canApply = $derived(!!edition && connected && !patchActive && ready);

  let sourceSeason = $derived(edition ? formatSeason(edition.startYear) : "—");
  let targetSeason = $derived.by(() => {
    if (!ready || modStartYear === null) return "—";
    return formatSeason(modStartYear);
  });

  function handleApply() {
    if (!canApply || shiftBack === null || modStartYear === null) return;
    onApply(shiftBack, modStartYear);
  }
</script>

<article class="card">
  <header class="head">
    <h2>Configuration</h2>
    <p>Edition is detected from the running fm.exe.</p>
  </header>

  <section class="block">
    <span class="label">Detected FM Edition</span>
    <div class="edition" class:edition--idle={!edition}>
      {#if edition}
        <div class="edition-text">
          <span class="edition-name">{edition.label}</span>
          <span class="edition-season">starts {sourceSeason}</span>
        </div>
        <span class="tag ok" title="Read from fm.exe">
          <span class="tag-dot"></span>
          Auto
        </span>
      {:else if connected}
        <span class="edition-idle">Identifying edition…</span>
      {:else}
        <span class="edition-idle">Waiting for fm.exe…</span>
      {/if}
    </div>
  </section>

  <NumberField
    label="Mod original start year"
    bind:value={modStartYear}
    placeholder="e.g. 1986"
    min={1900}
    max={2050}
    hint={shiftHint}
    error={modStartYearError}
    disabled={inputsLocked}
  />

  <PreviewCard {sourceSeason} {targetSeason} valid={ready} />

  <ActionBar
    {canApply}
    canRestore={connected && patchActive}
    {busy}
    onApply={handleApply}
    {onRestore}
  />
</article>

<style>
  .card {
    position: relative;
    background: var(--surface-card-tint), var(--surface-card);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: var(--accent-shadow);
    overflow: hidden;
  }

  .card::before {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    padding: 1px;
    background: var(--card-edge);
    -webkit-mask:
      linear-gradient(#000 0 0) content-box,
      linear-gradient(#000 0 0);
    -webkit-mask-composite: xor;
            mask-composite: exclude;
    pointer-events: none;
    opacity: 0.7;
  }

  .head {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .head h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
    letter-spacing: 0.2px;
  }

  .head p {
    margin: 0;
    font-size: 11px;
    color: var(--text-muted);
  }

  .block {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .edition {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--r-md);
    border: 1px solid var(--border);
    background: var(--surface-0);
  }

  .edition--idle {
    border-style: dashed;
    background: transparent;
  }

  .edition-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .edition-name {
    font-size: 14px;
    font-weight: 700;
    letter-spacing: 0.2px;
    background: var(--accent-text-grad);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }

  .edition-season {
    font-size: 10px;
    color: var(--text-muted);
    letter-spacing: 0.3px;
  }

  .edition-idle {
    font-size: 12px;
    color: var(--text-muted);
    font-style: italic;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.4px;
    padding: 3px 8px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-1);
    color: var(--text-muted);
  }

  .tag.ok {
    border-color: var(--ok-border, #2a6b3a);
    background: var(--ok-bg, rgba(42, 107, 58, 0.12));
    color: var(--ok-text, #5ec97a);
  }

  .tag-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
    opacity: 0.8;
  }
</style>
