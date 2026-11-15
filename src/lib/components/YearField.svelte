<script lang="ts">
  type Props = {
    label: string;
    value: number | null;
    placeholder?: string;
    /** Value used when stepping from an empty field. */
    fallback: number;
    min: number;
    max: number;
    hint?: string;
    error?: string | null;
    disabled?: boolean;
  };

  let {
    label,
    value = $bindable(),
    placeholder = "",
    fallback,
    min,
    max,
    hint = "",
    error = null,
    disabled = false,
  }: Props = $props();

  const inputId = `year-${Math.random().toString(36).slice(2, 8)}`;
  const msgId = `${inputId}-msg`;

  function onInput(e: Event) {
    const t = e.target as HTMLInputElement;
    // Keep only digits, max 4 — a year field never needs anything else.
    const digits = t.value.replace(/\D/g, "").slice(0, 4);
    if (digits !== t.value) t.value = digits;
    value = digits === "" ? null : Number(digits);
  }

  function step(delta: number) {
    if (disabled) return;
    const base = value ?? fallback - delta;
    value = Math.min(max, Math.max(min, base + delta));
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "ArrowUp") {
      e.preventDefault();
      step(e.shiftKey ? 10 : 1);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      step(e.shiftKey ? -10 : -1);
    }
  }
</script>

<div class="field" class:invalid={!!error} class:disabled>
  <label class="eyebrow" for={inputId}>{label}</label>

  <div class="control">
    <button
      type="button"
      class="step"
      tabindex="-1"
      aria-label="Previous year"
      {disabled}
      onclick={() => step(-1)}
    >
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <path d="M3.5 8h9" />
      </svg>
    </button>

    <input
      id={inputId}
      type="text"
      inputmode="numeric"
      autocomplete="off"
      spellcheck="false"
      maxlength="4"
      {placeholder}
      {disabled}
      value={value ?? ""}
      oninput={onInput}
      onkeydown={onKeydown}
      aria-invalid={!!error}
      aria-describedby={msgId}
    />

    <button
      type="button"
      class="step"
      tabindex="-1"
      aria-label="Next year"
      {disabled}
      onclick={() => step(1)}
    >
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <path d="M3.5 8h9M8 3.5v9" />
      </svg>
    </button>
  </div>

  <p class="msg" class:err={!!error} id={msgId}>
    {error ?? hint}
  </p>
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .control {
    display: flex;
    align-items: stretch;
    height: 44px;
    border: 1px solid var(--line-strong);
    border-radius: var(--r-md);
    background: var(--bg-raised);
    overflow: hidden;
    transition:
      border-color 140ms ease,
      box-shadow 140ms ease;
  }

  .control:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .invalid .control {
    border-color: var(--err-line);
  }

  .invalid .control:focus-within {
    border-color: var(--err);
    box-shadow: 0 0 0 3px var(--err-soft);
  }

  .disabled .control {
    background: var(--bg-sunken);
    border-color: var(--line);
  }

  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    text-align: center;
    font-family: var(--font-display);
    font-size: 20px;
    font-weight: 600;
    letter-spacing: 0.04em;
    font-variant-numeric: tabular-nums;
    color: var(--text);
    user-select: text;
  }

  input:focus-visible {
    outline: none;
  }

  input::placeholder {
    color: var(--text-3);
    font-weight: 400;
    opacity: 0.7;
  }

  input:disabled {
    color: var(--text-2);
    cursor: not-allowed;
  }

  .step {
    width: 44px;
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    transition:
      background-color 120ms ease,
      color 120ms ease;
  }

  .step:first-child {
    border-right: 1px solid var(--line);
  }

  .step:last-child {
    border-left: 1px solid var(--line);
  }

  .step:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text);
  }

  .step:active:not(:disabled) {
    background: var(--bg-active);
  }

  .step:disabled {
    color: var(--text-3);
    opacity: 0.5;
    cursor: not-allowed;
  }

  .msg {
    margin: 0;
    min-height: 18px;
    font-size: 12px;
    color: var(--text-3);
    transition: color 140ms ease;
  }

  .msg.err {
    color: var(--err);
  }
</style>
