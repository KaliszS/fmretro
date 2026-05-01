<script lang="ts">
  type Props = {
    sourceSeason: string;
    targetSeason: string;
    valid: boolean;
  };

  let { sourceSeason, targetSeason, valid }: Props = $props();
</script>

<div class="preview" class:idle={!valid}>
  <div class="row">
    <div class="block source">
      <span class="label">Game shows</span>
      <span class="value">{valid ? sourceSeason : "—"}</span>
    </div>
    <div class="arrow" aria-hidden="true">
      <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M5 12h14M13 6l6 6-6 6" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </div>
    <div class="block target">
      <span class="label">You see</span>
      <span class="value">{valid ? targetSeason : "—"}</span>
    </div>
  </div>
</div>

<style>
  .preview {
    border-radius: var(--r-md);
    padding: 12px 14px;
    background: var(--accent-card-bg), var(--surface-1);
    border: 1px solid var(--accent-card-border);
    transition: opacity 0.2s, filter 0.2s;
  }

  .preview.idle {
    opacity: 0.5;
    filter: saturate(0.4);
  }

  .row {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 12px;
  }

  .block {
    display: flex;
    flex-direction: column;
    gap: 2px;
    align-items: center;
    text-align: center;
  }

  .label {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 1.2px;
    color: var(--text-muted);
    font-weight: 600;
  }

  .value {
    font-size: 22px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.5px;
  }

  .target .value {
    background: var(--accent-grad);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }

  .arrow {
    color: var(--text-muted);
    display: grid;
    place-items: center;
  }
</style>
