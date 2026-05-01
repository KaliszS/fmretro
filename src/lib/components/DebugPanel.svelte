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
  let timer: ReturnType<typeof setInterval> | null = null;

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
  });

  function copyAll() {
    if (!info) return;
    const lines: string[] = [];
    lines.push(`# FM Retro Patcher debug — v${info.patcher_version}`);
    lines.push(`host: ${info.host_os}/${info.host_arch}`);
    lines.push("");
    if (info.process) {
      lines.push(`process.pid: ${info.process.pid}`);
      lines.push(`process.path: ${info.process.path ?? "(unknown)"}`);
      if (info.process.fm_module) {
        lines.push(
          `fm.exe: base=${info.process.fm_module.base} size=${info.process.fm_module.size} (${info.process.fm_module.size_mb.toFixed(2)} MB)`,
        );
      }
      if (info.process.vcrt_module) {
        lines.push(
          `VCRUNTIME140.dll: base=${info.process.vcrt_module.base} size=${info.process.vcrt_module.size} (${info.process.vcrt_module.size_mb.toFixed(2)} MB)`,
        );
      }
    } else {
      lines.push("process: not connected");
    }
    lines.push("");
    if (info.table) {
      lines.push(`year_table.base: ${info.table.base}`);
      lines.push(`year_table.last_entry: ${info.table.last_entry}`);
      lines.push(
        `year_table: stride=0x${info.table.stride.toString(16)} count=${info.table.entry_count} range=${info.table.year_first}..${info.table.year_last}`,
      );
    } else {
      lines.push("year_table: not located");
    }
    lines.push("");
    if (info.patch) {
      lines.push(
        `patch: shift=${info.patch.shift > 0 ? "+" : ""}${info.patch.shift} saved=${info.patch.saved_entries}`,
      );
    } else {
      lines.push("patch: idle");
    }
    lines.push("");
    if (info.hooks.length) {
      lines.push("hooks:");
      for (const h of info.hooks) {
        lines.push(`  ${h.kind} [${h.is_active ? "active" : "stale"}]`);
        lines.push(`    module: ${h.module} · leaf: ${h.leaf}`);
        lines.push(
          `    hook=${h.hook_addr}${h.hook_rva ? ` (RVA ${h.hook_rva})` : ""}`,
        );
        lines.push(`    shellcode=${h.shellcode_addr} size=${h.shellcode_size} B`);
        lines.push(`    rel32 distance=${h.jump_distance}`);
        if (h.current_bytes) {
          lines.push(`    live bytes: ${h.current_bytes}`);
        }
      }
    } else {
      lines.push("hooks: none");
    }
    lines.push("");
    lines.push(`last_message: ${info.last_message || "(none)"}`);
    navigator.clipboard?.writeText(lines.join("\n")).catch(() => {});
  }

  function fmtAddr(s: string) {
    return s;
  }
</script>

<article class="card">
  <header class="head">
    <div class="head-text">
      <h2>Debug</h2>
      <p>Live runtime state. Updates every 1.5 s.</p>
    </div>
    <div class="head-actions">
      <button type="button" class="ghost-btn" onclick={copyAll} title="Copy diagnostic dump">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="11" height="11" rx="2" />
          <path d="M5 15V5a2 2 0 0 1 2-2h10" />
        </svg>
        Copy
      </button>
      <button type="button" class="ghost-btn" onclick={onBack} title="Back to configuration">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M15 18l-6-6 6-6" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        Back
      </button>
    </div>
  </header>

  {#if error}
    <p class="err">Failed to read debug info: {error}</p>
  {/if}

  {#if info}
    <!-- Host -->
    <section class="group">
      <span class="label">Host</span>
      <div class="kv">
        <div class="row"><dt class="k">Patcher</dt><dd class="v mono">v{info.patcher_version}</dd></div>
        <div class="row"><dt class="k">OS / arch</dt><dd class="v">{info.host_os} · {info.host_arch}</dd></div>
      </div>
    </section>

    <!-- Process -->
    <section class="group">
      <span class="label">Process</span>
      {#if info.process}
        <div class="kv">
          <div class="row">
            <dt class="k">PID</dt>
            <dd class="v mono">{info.process.pid}</dd>
          </div>
          {#if info.process.path}
            <div class="row">
              <dt class="k">Path</dt>
              <dd class="v path" title={info.process.path}>{info.process.path}</dd>
            </div>
          {/if}
          {#if info.process.fm_module}
            <div class="row">
              <dt class="k">fm.exe</dt>
              <dd class="v">
                <span class="mono">{fmtAddr(info.process.fm_module.base)}</span>
                <span class="dim">· {info.process.fm_module.size_mb.toFixed(2)} MB</span>
              </dd>
            </div>
          {/if}
          {#if info.process.vcrt_module}
            <div class="row">
              <dt class="k">VCRUNTIME140</dt>
              <dd class="v">
                <span class="mono">{fmtAddr(info.process.vcrt_module.base)}</span>
                <span class="dim">· {info.process.vcrt_module.size_mb.toFixed(2)} MB</span>
              </dd>
            </div>
          {/if}
        </div>
      {:else}
        <p class="muted">fm.exe is not running.</p>
      {/if}
    </section>

    <!-- Year table -->
    <section class="group">
      <span class="label">Year string table</span>
      {#if info.table}
        <div class="kv">
          <div class="row">
            <dt class="k">Base</dt>
            <dd class="v mono">{fmtAddr(info.table.base)}</dd>
          </div>
          <div class="row">
            <dt class="k">Last entry</dt>
            <dd class="v mono">{fmtAddr(info.table.last_entry)}</dd>
          </div>
          <div class="row">
            <dt class="k">Layout</dt>
            <dd class="v">
              stride <span class="mono">0x{info.table.stride.toString(16).toUpperCase()}</span>
              <span class="dim">·</span>
              {info.table.entry_count} entries
              <span class="dim">·</span>
              {info.table.year_first}–{info.table.year_last}
            </dd>
          </div>
        </div>
      {:else}
        <p class="muted">Not located. Apply a patch first or connect to fm.exe.</p>
      {/if}
    </section>

    <!-- Patch -->
    <section class="group">
      <span class="label">Year patch</span>
      {#if info.patch}
        <div class="kv">
          <div class="row">
            <dt class="k">Shift</dt>
            <dd class="v mono">
              {info.patch.shift > 0 ? "+" : ""}{info.patch.shift} years
            </dd>
          </div>
          <div class="row">
            <dt class="k">Saved entries</dt>
            <dd class="v">{info.patch.saved_entries}</dd>
          </div>
        </div>
      {:else}
        <p class="muted">No active patch.</p>
      {/if}
    </section>

    <!-- Hooks -->
    <section class="group">
      <span class="label">Installed hooks</span>
      {#if info.hooks.length}
        <ul class="hooks">
          {#each info.hooks as h}
            <li class="hook">
              <div class="hook-head">
                <span class="hook-name">{h.kind}</span>
                <span class="tag" class:ok={h.is_active}>
                  <span class="tag-dot"></span>
                  {h.is_active ? "active" : "stale"}
                </span>
              </div>
              <div class="hook-meta">
                <span class="dim">{h.module} · {h.leaf}</span>
              </div>
              <div class="hook-grid">
                <span class="hk">Hook site</span>
                <span class="hv">
                  <span class="mono">{h.hook_addr}</span>
                  {#if h.hook_rva}
                    <span class="dim">· RVA {h.hook_rva}</span>
                  {/if}
                </span>

                <span class="hk">Shellcode</span>
                <span class="hv">
                  <span class="mono">{h.shellcode_addr}</span>
                  <span class="dim">· {h.shellcode_size} B</span>
                </span>

                <span class="hk">JMP rel32</span>
                <span class="hv">
                  <span class="mono">
                    {h.jump_distance >= 0 ? "+" : ""}{h.jump_distance.toLocaleString()}
                  </span>
                  <span class="dim">bytes</span>
                </span>

                {#if h.current_bytes}
                  <span class="hk">Live bytes</span>
                  <span class="hv mono small">{h.current_bytes}</span>
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="muted">No hooks installed.</p>
      {/if}
    </section>

    {#if info.last_message}
      <section class="group">
        <span class="label">Last message</span>
        <p class="msg">{info.last_message}</p>
      </section>
    {/if}
  {:else if !error}
    <p class="muted">Loading…</p>
  {/if}
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
  }

  .head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 10px;
  }

  .head-text h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
  }

  .head-text p {
    margin: 1px 0 0;
    font-size: 11px;
    color: var(--text-muted);
  }

  .head-actions {
    display: flex;
    gap: 6px;
  }

  .ghost-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 9px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-soft);
    background: var(--surface-1);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s, background 0.15s;
  }

  .ghost-btn:hover {
    color: var(--text);
    border-color: var(--border-strong);
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .kv {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: var(--surface-0);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    padding: 6px 10px;
  }

  .row {
    display: grid;
    grid-template-columns: 110px 1fr;
    gap: 10px;
    align-items: baseline;
    padding: 3px 0;
  }

  .row + .row {
    border-top: 1px dashed var(--border);
  }

  .k {
    margin: 0;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--text-muted);
  }

  .v {
    margin: 0;
    font-size: 12px;
    color: var(--text);
    word-break: break-all;
  }

  .v.path {
    font-size: 11px;
    color: var(--text-soft);
    user-select: text;
  }

  .mono {
    font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
    color: var(--accent-a);
    user-select: text;
  }

  .dim {
    color: var(--text-muted);
    font-size: 11px;
  }

  .muted {
    margin: 0;
    color: var(--text-muted);
    font-size: 12px;
    font-style: italic;
  }

  .err {
    margin: 0;
    color: var(--err);
    font-size: 12px;
  }

  .hooks {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .hook {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--surface-0);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    padding: 8px 10px;
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
    color: var(--text);
  }

  .hook-name::before {
    content: "›";
    color: var(--accent-b);
    margin-right: 6px;
  }

  .hook-meta {
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .hook-grid {
    display: grid;
    grid-template-columns: 90px 1fr;
    gap: 3px 10px;
    font-size: 11px;
    align-items: baseline;
  }

  .hk {
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.6px;
    font-size: 9.5px;
  }

  .hv {
    color: var(--text);
    word-break: break-all;
  }

  .hv.small {
    font-size: 10.5px;
  }

  .msg {
    margin: 0;
    padding: 8px 10px;
    background: var(--surface-0);
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    font-size: 11px;
    color: var(--text-soft);
    user-select: text;
  }
</style>
