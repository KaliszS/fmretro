<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import "$lib/styles/theme.css";
  import { detectFm, getStatus, applyPatch, restorePatch } from "$lib/api";
  import type { Status } from "$lib/types";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import TopBar from "$lib/components/TopBar.svelte";
  import ConfigPanel from "$lib/components/ConfigPanel.svelte";
  import DebugPanel from "$lib/components/DebugPanel.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import VersionBadge from "$lib/components/VersionBadge.svelte";

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
  let modStartYear = $state<number | null>(null);
  let toast = $state<{ kind: "info" | "ok" | "err"; text: string } | null>(
    null,
  );
  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  function flash(kind: "info" | "ok" | "err", text: string, ms = 4000) {
    toast = { kind, text };
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toast = null;
    }, ms);
  }

  async function refresh() {
    try {
      await detectFm();
      status = await getStatus();
    } catch (e) {
      console.warn("detect failed", e);
    }
  }

  async function handleApply(shiftBack: number, minYear: number) {
    if (busy) return;
    busy = true;
    try {
      await applyPatch(shiftBack, minYear);
      flash("ok", `Retro patch applied · shift ${shiftBack} years`);
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
      flash("ok", "Original years restored");
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

<div class="shell">
  <TitleBar />
  <main class="app">
    <TopBar
      connected={status.fm_detected}
      debugActive={view === "debug"}
      onToggleDebug={toggleDebug}
    />

    {#if view === "main"}
      <ConfigPanel
        connected={status.fm_detected}
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

    <div class="toast-slot">
      {#if toast}
        <Toast kind={toast.kind} text={toast.text} />
      {/if}
    </div>

    <footer class="foot">
      Closing this window restores all patches automatically.
    </footer>
  </main>
  <VersionBadge />
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
    overflow: hidden;
  }

  .shell {
    position: relative;
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: var(--r-lg);
    background:
      radial-gradient(900px 500px at 85% -10%, var(--backdrop-glow-a), transparent 60%),
      radial-gradient(800px 600px at -10% 110%, var(--backdrop-glow-b), transparent 60%),
      linear-gradient(180deg, var(--bg-page-a) 0%, var(--bg-page-b) 100%);
    box-shadow: 0 30px 70px -20px rgba(0, 0, 0, 0.55);
  }

  .app {
    position: relative;
    z-index: 1;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 14px 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    scrollbar-width: thin;
    scrollbar-color: var(--border-strong) transparent;
  }

  .app::-webkit-scrollbar {
    width: 6px;
  }
  .app::-webkit-scrollbar-thumb {
    background: var(--border-strong);
    border-radius: 999px;
  }
  .app::-webkit-scrollbar-track {
    background: transparent;
  }

  .toast-slot {
    min-height: 26px;
    display: flex;
    justify-content: center;
  }

  .foot {
    text-align: center;
    color: var(--text-muted);
    font-size: 10px;
    padding: 2px 0 0;
    letter-spacing: 0.2px;
  }
</style>
