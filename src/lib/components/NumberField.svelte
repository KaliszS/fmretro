<script lang="ts">
  type Props = {
    label: string;
    value: number | null;
    placeholder?: string;
    min?: number;
    max?: number;
    hint?: string;
    error?: string | null;
    disabled?: boolean;
  };

  let {
    label,
    value = $bindable(),
    placeholder = "",
    min,
    max,
    hint = "",
    error = null,
    disabled = false,
  }: Props = $props();

  const inputId = `f-${Math.random().toString(36).slice(2, 8)}`;

  function onInput(e: Event) {
    const t = e.target as HTMLInputElement;
    if (t.value === "") {
      value = null;
      return;
    }
    const n = Number(t.value);
    value = Number.isNaN(n) ? null : n;
  }
</script>

<div class="field" class:error={!!error} class:disabled>
  <label class="label" for={inputId}>{label}</label>
  <div class="input-wrap">
    <input
      id={inputId}
      type="number"
      inputmode="numeric"
      {placeholder}
      {min}
      {max}
      {disabled}
      value={value ?? ""}
      oninput={onInput}
    />
  </div>
  {#if error}
    <p class="msg err">{error}</p>
  {:else if hint}
    <p class="msg">{hint}</p>
  {/if}
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .input-wrap {
    position: relative;
  }

  input {
    width: 100%;
    background: var(--surface-input);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: 10px 12px;
    font-size: 14px;
    font-variant-numeric: tabular-nums;
    outline: none;
    transition: border-color 0.15s, box-shadow 0.15s, background 0.15s;
  }

  input::placeholder {
    color: var(--text-muted);
    opacity: 0.6;
  }

  input:focus {
    border-color: var(--focus-border);
    box-shadow: 0 0 0 3px var(--focus-ring);
    background: var(--focus-bg);
  }

  input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .field.error input {
    border-color: var(--err-border);
    box-shadow: 0 0 0 3px var(--err-focus);
  }

  /* Hide spinner buttons */
  input::-webkit-outer-spin-button,
  input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  input[type="number"] {
    -moz-appearance: textfield;
  }

  .msg {
    margin: 0;
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.4;
  }

  .msg.err {
    color: var(--err);
  }
</style>
