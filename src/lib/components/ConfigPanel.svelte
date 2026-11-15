<script lang="ts">
  import GameCard from "./GameCard.svelte";
  import YearField from "./YearField.svelte";
  import SeasonBoard from "./SeasonBoard.svelte";
  import ActionBar from "./ActionBar.svelte";
  import { findEdition, formatSeason, editionIdForMajor } from "../editions";
  import type { FmEditionInfo } from "../types";

  type Props = {
    connected: boolean;
    pid: number | null;
    patchActive: boolean;
    busy: boolean;
    detectedEdition: FmEditionInfo | null;
    modStartYear: number | null;
    onApply: (shiftBack: number, minYear: number) => void;
    onRestore: () => void;
  };

  let {
    connected,
    pid,
    patchActive,
    busy,
    detectedEdition,
    modStartYear = $bindable(),
    onApply,
    onRestore,
  }: Props = $props();

  const DEFAULT_YEAR = 1986;

  let edition = $derived.by(() => {
    const id = detectedEdition ? editionIdForMajor(detectedEdition.major) : null;
    return id ? findEdition(id) : null;
  });

  let inputsLocked = $derived(patchActive || busy || !edition);

  let shiftBack = $derived(
    edition && modStartYear !== null ? edition.startYear - modStartYear : null,
  );

  let error = $derived.by<string | null>(() => {
    if (modStartYear === null) return null;
    // Partial input while typing — don't nag until four digits are in.
    if (modStartYear < 1000) return null;
    if (modStartYear < 1900 || modStartYear > 2050) return "Use a year between 1900 and 2050.";
    if (edition && modStartYear >= edition.startYear)
      return `Must be before ${edition.startYear}, when this edition starts.`;
    if (edition && edition.startYear - modStartYear > 100) return "That's more than 100 years back.";
    return null;
  });

  let ready = $derived(
    modStartYear !== null &&
      modStartYear >= 1000 &&
      error === null &&
      shiftBack !== null &&
      shiftBack > 0,
  );

  let hint = $derived.by(() => {
    if (!edition) return "Available once the game is detected.";
    if (patchActive) return "Restore the original years to change this.";
    if (ready && shiftBack !== null) return `All dates shift ${shiftBack} years back.`;
    return "The first season of your historical database.";
  });

  let canApply = $derived(!!edition && connected && !patchActive && ready);

  let sourceSeason = $derived(edition ? formatSeason(edition.startYear) : null);
  let targetSeason = $derived(ready && modStartYear !== null ? formatSeason(modStartYear) : null);

  function handleApply() {
    if (!canApply || shiftBack === null || modStartYear === null) return;
    onApply(shiftBack, modStartYear);
  }
</script>

<div class="panel">
  <GameCard {connected} {edition} {pid} {patchActive} />

  <YearField
    label="Mod start year"
    bind:value={modStartYear}
    placeholder={String(DEFAULT_YEAR)}
    fallback={DEFAULT_YEAR}
    min={1900}
    max={edition ? edition.startYear - 1 : 2050}
    {hint}
    {error}
    disabled={inputsLocked}
  />

  <SeasonBoard source={sourceSeason} target={targetSeason} shift={ready ? shiftBack : null} />

  <ActionBar
    {patchActive}
    {canApply}
    canRestore={connected && patchActive}
    {busy}
    onApply={handleApply}
    {onRestore}
  />
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
</style>
