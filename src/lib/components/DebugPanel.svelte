<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getDebug } from "../api";
  import type { DebugInfo } from "../types";

  type Props = {
    onBack: () => void;
  };

  let { onBack }: Props = $props();

  let info = $state<DebugInfo | null>(null);
  let error = $state<string | null>(null);
  let copied = $state(false);
  let timer: ReturnType<typeof setInterval> | null = null;
  let copiedTimer: ReturnType<typeof setTimeout> | null = null;

  async function refresh() {
    try {
      info = await getDebug();
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  onMount(() => {
    refresh();
    timer = setInterval(refresh, 1500);
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
    if (copiedTimer) clearTimeout(copiedTimer);
  });

  function dump(d: DebugInfo): string {
    const lines: string[] = [];
    lines.push(`# FM Retro Patcher debug — v${d.patcher_version}`);
    lines.push(`host: ${d.host_os}/${d.host_arch}`);
    lines.push("");
    if (d.process) {
      lines.push(`process.pid: ${d.process.pid}`);
      lines.push(`process.path: ${d.process.path ?? "(unknown)"}`);
      if (d.process.fm_module) {
        const m = d.process.fm_module;
        lines.push(`fm.exe: base=${m.base} size=${m.size} (${m.size_mb.toFixed(2)} MB)`);
      }
      if (d.process.vcrt_module) {
        const m = d.process.vcrt_module;
        lines.push(`VCRUNTIME140.dll: base=${m.base} size=${m.size} (${m.size_mb.toFixed(2)} MB)`);
      }
    } else {
      lines.push("process: not connected");
    }
    lines.push("");
    if (d.table) {
      const t = d.table;
      lines.push(`year_table.base: ${t.base}`);
      lines.push(`year_table.last_entry: ${t.last_entry}`);
      lines.push(
        `year_table: stride=0x${t.stride.toString(16)} count=${t.entry_count} range=${t.year_first}..${t.year_last}`,
      );
      if (t.season_base) lines.push(`season_table.base: ${t.season_base}`);
    } else {
      lines.push("year_table: not located");
    }
    lines.push("");
    if (d.patch) {
      lines.push(
        `patch: shift=${d.patch.shift > 0 ? "+" : ""}${d.patch.shift} saved=${d.patch.saved_entries}`,
      );
    } else {
      lines.push("patch: idle");
    }
    lines.push("");
    if (d.hooks.length) {
      lines.push("hooks:");
      for (const h of d.hooks) {
        lines.push(`  ${h.kind} [${h.is_active ? "active" : "stale"}]`);
        lines.push(`    module: ${h.module} · leaf: ${h.leaf}`);
        lines.push(`    hook=${h.hook_addr}${h.hook_rva ? ` (RVA ${h.hook_rva})` : ""}`);
        lines.push(`    shellcode=${h.shellcode_addr} size=${h.shellcode_size} B`);
        lines.push(`    rel32 distance=${h.jump_distance}`);
        if (h.current_bytes) lines.push(`    live bytes: ${h.current_bytes}`);
      }
    } else {
      lines.push("hooks: none");
    }
    lines.push("");
    lines.push(`last_message: ${d.last_message || "(none)"}`);
    return lines.join("\n");
  }

  async function copyAll() {
    if (!info) return;
    try {
      await navigator.clipboard.writeText(dump(info));
      copied = true;
      if (copiedTimer) clearTimeout(copiedTimer);
      copiedTimer = setTimeout(() => (copied = false), 1600);
    } catch {
      // Clipboard can be unavailable; nothing useful to show here.
    }
  }
</script>

<div class="debug">
  <div class="head">
    <button type="button" class="back" onclick={onBack}>
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M10 3.5L5.5 8l4.5 4.5" />
      </svg>
      Diagnostics
    </button>
    <button type="button" class="copy" onclick={copyAll} disabled={!info}>
      {#if copied}
        <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M3.5 8.5l3 3 6-7" />
        </svg>
        Copied
      {:else}
        <svg viewBox="0 0 16 16" width="13" height="13" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true">
          <rect x="5.5" y="5.5" width="8" height="8" rx="1.5" />
          <path d="M10.5 3.5v-.5a1 1 0 0 0-1-1h-6a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h.5" />
        </svg>
        Copy report
      {/if}
    </button>
  </div>

  {#if error}
    <p class="err">Couldn't read diagnostics: {error}</p>
  {/if}

  {#if info}
    <section>
      <h3>Host</h3>
      <dl>
        <dt>Patcher</dt>
        <dd class="mono">v{info.patcher_version}</dd>
        <dt>Platform</dt>
        <dd>{info.host_os} · {info.host_arch}</dd>
      </dl>
    </section>

    <section>
      <h3>Process</h3>
      {#if info.process}
        <dl>
          <dt>PID</dt>
          <dd class="mono">{info.process.pid}</dd>
          {#if info.process.path}
            <dt>Path</dt>
            <dd class="path" title={info.process.path}>{info.process.path}</dd>
          {/if}
          {#if info.process.fm_module}
            <dt>fm.exe</dt>
            <dd>
              <span class="mono">{info.process.fm_module.base}</span>
              <span class="dim">{info.process.fm_module.size_mb.toFixed(2)} MB</span>
            </dd>
          {/if}
          {#if info.process.vcrt_module}
            <dt>VCRUNTIME140</dt>
            <dd>
              <span class="mono">{info.process.vcrt_module.base}</span>
              <span class="dim">{info.process.vcrt_module.size_mb.toFixed(2)} MB</span>
            </dd>
          {/if}
        </dl>
      {:else}
        <p class="empty">Football Manager is not running.</p>
      {/if}
    </section>

    <section>
      <h3>Year string table</h3>
      {#if info.table}
        <dl>
          <dt>Base</dt>
          <dd class="mono">{info.table.base}</dd>
          <dt>Last entry</dt>
          <dd class="mono">{info.table.last_entry}</dd>
          <dt>Layout</dt>
          <dd>
            <span class="mono">0x{info.table.stride.toString(16).toUpperCase()}</span>
            <span class="dim">stride</span>
            · {info.table.entry_count} entries · {info.table.year_first}–{info.table.year_last}
          </dd>
          {#if info.table.season_base}
            <dt>Season base</dt>
            <dd class="mono">{info.table.season_base}</dd>
          {/if}
        </dl>
      {:else}
        <p class="empty">Not located yet — it's found when the game is connected.</p>
      {/if}
    </section>

    <section>
      <h3>Year patch</h3>
      {#if info.patch}
        <dl>
          <dt>Shift</dt>
          <dd class="mono">{info.patch.shift > 0 ? "+" : ""}{info.patch.shift} years</dd>
          <dt>Saved entries</dt>
          <dd class="mono">{info.patch.saved_entries}</dd>
        </dl>
      {:else}
        <p class="empty">No active patch.</p>
      {/if}
    </section>

    <section>
      <h3>Hooks</h3>
      {#if info.hooks.length}
        <ul class="hooks">
          {#each info.hooks as h (h.kind + h.hook_addr)}
            <li>
              <div class="hook-head">
                <span class="hook-name">{h.kind}</span>
                <span class="state" class:on={h.is_active}>{h.is_active ? "Active" : "Stale"}</span>
              </div>
              <span class="dim">{h.module} · {h.leaf}</span>
              <dl class="tight">
                <dt>Hook site</dt>
                <dd>
                  <span class="mono">{h.hook_addr}</span>
                  {#if h.hook_rva}<span class="dim">RVA {h.hook_rva}</span>{/if}
                </dd>
                <dt>Shellcode</dt>
                <dd>
                  <span class="mono">{h.shellcode_addr}</span>
                  <span class="dim">{h.shellcode_size} B</span>
                </dd>
                <dt>JMP rel32</dt>
                <dd class="mono">{h.jump_distance >= 0 ? "+" : ""}{h.jump_distance.toLocaleString()}</dd>
                {#if h.current_bytes}
                  <dt>Live bytes</dt>
                  <dd class="mono small">{h.current_bytes}</dd>
                {/if}
              </dl>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="empty">No hooks installed.</p>
      {/if}
    </section>

    {#if info.last_message}
      <section>
        <h3>Last message</h3>
        <p class="message">{info.last_message}</p>
      </section>
    {/if}
  {:else if !error}
    <p class="empty">Loading…</p>
  {/if}
</div>

<style>
  .debug {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .back,
  .copy {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--r-sm);
    background: transparent;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      color 120ms ease;
  }

  .back {
    margin-left: -8px;
    padding-left: 6px;
    font-family: var(--font-display);
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .copy {
    font-size: 12px;
    color: var(--text-2);
    border: 1px solid var(--line-strong);
  }

  .back:hover,
  .copy:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text);
  }

  .copy:disabled {
    opacity: 0.5;
    cursor: default;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  h3 {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-3);
  }

  dl {
    margin: 0;
    display: grid;
    grid-template-columns: 104px 1fr;
    column-gap: 12px;
    padding: 2px 12px;
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    background: var(--bg-raised);
  }

  dt,
  dd {
    margin: 0;
    padding: 7px 0;
    font-size: 12px;
    border-top: 1px solid var(--line);
  }

  dt:first-of-type,
  dt:first-of-type + dd {
    border-top: 0;
  }

  dt {
    color: var(--text-2);
  }

  dd {
    color: var(--text);
    min-width: 0;
    overflow-wrap: anywhere;
    user-select: text;
  }

  dd .mono + .dim {
    margin-left: 6px;
  }

  .mono {
    font-size: 11.5px;
  }

  .small {
    font-size: 10.5px;
    color: var(--text-2);
  }

  .dim {
    color: var(--text-3);
    font-size: 11px;
  }

  .path {
    font-size: 11px;
    color: var(--text-2);
  }

  .empty {
    margin: 0;
    padding: 10px 12px;
    border: 1px dashed var(--line-strong);
    border-radius: var(--r-md);
    font-size: 12px;
    color: var(--text-3);
  }

  .err {
    margin: 0;
    padding: 10px 12px;
    border-radius: var(--r-md);
    background: var(--err-soft);
    color: var(--err);
    font-size: 12px;
  }

  .hooks {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .hooks li {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px;
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    background: var(--bg-raised);
  }

  .hook-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .hook-name {
    font-size: 12px;
    font-weight: 600;
  }

  .state {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 999px;
    color: var(--text-3);
    background: var(--bg-active);
  }

  .state.on {
    color: var(--ok);
    background: var(--ok-soft);
  }

  dl.tight {
    margin-top: 4px;
    padding: 0;
    border: 0;
    background: transparent;
    grid-template-columns: 80px 1fr;
  }

  dl.tight dt,
  dl.tight dd {
    padding: 4px 0;
    border-top: 0;
    font-size: 11.5px;
  }

  .message {
    margin: 0;
    padding: 10px 12px;
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    background: var(--bg-raised);
    font-size: 12px;
    color: var(--text-2);
    user-select: text;
  }
</style>
