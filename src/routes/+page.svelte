<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { fade } from "svelte/transition";
  import "$lib/styles/theme.css";
  import { detectFm, getStatus, applyPatch, restorePatch } from "$lib/api";
  import type { Status } from "$lib/types";
  import { rise, dur } from "$lib/motion";
  import AppHeader from "$lib/components/AppHeader.svelte";
  import ConfigPanel from "$lib/components/ConfigPanel.svelte";
  import DebugPanel from "$lib/components/DebugPanel.svelte";
  import Toast from "$lib/components/Toast.svelte";

  const version: string = __APP_VERSION__;
  const YEAR_KEY = "fmretro:modStartYear";

  type ToastMsg = { id: number; kind: "info" | "ok" | "err"; text: string };

  let status = $state<Status>({
    fm_detected: false,
    pid: null,
    patch_active: false,
    shift: null,
    min_year: null,
    message: "",
    year_table_base: null,
    hooks: [],
    edition: null,
  });

  let busy = $state(false);
  let view = $state<"main" | "debug">("main");
  let modStartYear = $state<number | null>(loadYear());
  let toast = $state<ToastMsg | null>(null);
  let toastSeq = 0;
  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  function loadYear(): number | null {
    try {
      const raw = localStorage.getItem(YEAR_KEY);
      const n = raw ? Number(raw) : NaN;
      return Number.isInteger(n) && n >= 1900 && n <= 2050 ? n : null;
    } catch {
      return null;
    }
  }

  // Remember the last complete year so the next session starts pre-filled.
  $effect(() => {
    if (modStartYear === null || modStartYear < 1900) return;
    try {
      localStorage.setItem(YEAR_KEY, String(modStartYear));
    } catch {
      // Non-essential convenience.
    }
  });

  function flash(kind: ToastMsg["kind"], text: string, ms = kind === "err" ? 8000 : 3500) {
    toast = { id: ++toastSeq, kind, text };
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = null), ms);
  }

  function dismissToast() {
    if (toastTimer) clearTimeout(toastTimer);
    toast = null;
  }

  async function refresh() {
    try {
      await detectFm();
      status = await getStatus();
      // When the patch is already active, reflect the year it was applied with.
      if (status.patch_active && status.min_year !== null) {
        modStartYear = status.min_year;
      }
    } catch (e) {
      console.warn("detect failed", e);
    }
  }

  async function handleApply(shiftBack: number, minYear: number) {
    if (busy) return;
    busy = true;
    try {
      await applyPatch(shiftBack, minYear);
      flash("ok", `Patch applied — dates shifted ${shiftBack} years back.`);
    } catch (e) {
      flash("err", String(e));
    } finally {
      await refresh();
      busy = false;
    }
  }

  async function handleRestore() {
    if (busy) return;
    busy = true;
    try {
      await restorePatch();
      flash("ok", "Original years restored.");
    } catch (e) {
      flash("err", String(e));
    } finally {
      await refresh();
      busy = false;
    }
  }

  function toggleDebug() {
    view = view === "debug" ? "main" : "debug";
  }

  onMount(() => {
    refresh();
    pollHandle = setInterval(refresh, 2000);
  });

  onDestroy(() => {
    if (pollHandle) clearInterval(pollHandle);
    if (toastTimer) clearTimeout(toastTimer);
  });
</script>

<div class="window">
  <AppHeader debugActive={view === "debug"} onToggleDebug={toggleDebug} />

  <main class="scroll">
    <div class="stack">
      {#key view}
        <div class="view" in:rise={{ duration: 240, delay: 60 }} out:fade={{ duration: dur(100) }}>
          {#if view === "main"}
            <ConfigPanel
              connected={status.fm_detected}
              pid={status.pid}
              patchActive={status.patch_active}
              {busy}
              detectedEdition={status.edition}
              bind:modStartYear
              onApply={handleApply}
              onRestore={handleRestore}
            />
          {:else}
            <DebugPanel onBack={() => (view = "main")} />
          {/if}
        </div>
      {/key}
    </div>
  </main>

  <footer class="foot">
    <span>Closing this window restores the game automatically.</span>
    <span class="mono">v{version}</span>
  </footer>

  <div class="toasts" aria-live="polite">
    {#if toast}
      {#key toast.id}
        <Toast kind={toast.kind} text={toast.text} onDismiss={dismissToast} />
      {/key}
    {/if}
  </div>
</div>

<style>
  .window {
    position: relative;
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: var(--r-lg);
    border: 1px solid var(--line-strong);
    background: var(--bg);
    transition: background-color 200ms ease;
  }

  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 20px 20px 24px;
    scrollbar-width: thin;
    scrollbar-color: var(--line-strong) transparent;
    scrollbar-gutter: stable;
  }

  .scroll::-webkit-scrollbar {
    width: 8px;
  }

  .scroll::-webkit-scrollbar-thumb {
    background: var(--line-strong);
    border-radius: 999px;
    border: 2px solid var(--bg);
  }

  .scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  /* Views share one grid cell so the outgoing and incoming view overlap
     during the transition instead of stacking and jumping. */
  .stack {
    display: grid;
  }

  .view {
    grid-area: 1 / 1;
    min-width: 0;
  }

  .foot {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 20px;
    border-top: 1px solid var(--line);
    font-size: 11px;
    color: var(--text-3);
  }

  .foot .mono {
    font-size: 10.5px;
  }

  .toasts {
    position: absolute;
    left: 16px;
    right: 16px;
    bottom: 48px;
    display: grid;
    justify-items: center;
    pointer-events: none;
    z-index: 10;
  }

  /* A replaced toast fades out in place while the next one rises in. */
  .toasts > :global(*) {
    grid-area: 1 / 1;
  }
</style>
