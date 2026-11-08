<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { i18n, t } from "./lib/i18n.svelte";
  import { project } from "./lib/project.svelte";
  import DropZone from "./components/DropZone.svelte";
  import Preview from "./components/Preview.svelte";
  import Timeline from "./components/Timeline.svelte";
  import SettingsPanel from "./components/SettingsPanel.svelte";
  import ExportBar from "./components/ExportBar.svelte";

  let dragOver = $state(false);

  const VIDEO_EXT = ["mp4", "mov", "m4v"];

  async function openDialog() {
    const picked = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Video", extensions: VIDEO_EXT }],
    });
    if (typeof picked === "string") project.load(picked);
  }

  onMount(() => {
    document.documentElement.lang = i18n.locale;
    const unlisten = getCurrentWebview().onDragDropEvent((e) => {
      const p = e.payload;
      if (p.type === "enter" || p.type === "over") dragOver = true;
      else if (p.type === "leave") dragOver = false;
      else if (p.type === "drop") {
        dragOver = false;
        const file = p.paths.find((f) => VIDEO_EXT.some((x) => f.toLowerCase().endsWith("." + x)));
        if (file) project.load(file);
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  });

  // Re-estimate whenever anything affecting output changes.
  $effect(() => {
    void project.paramsKey;
    project.scheduleEstimate();
  });

  function onKeydown(e: KeyboardEvent) {
    const el = e.target as HTMLElement;
    if (!project.info || el.closest("input, select, textarea")) return;
    switch (e.key) {
      case " ":
        project.togglePlay();
        break;
      case "i":
      case "I":
        project.setStart(project.currentTime);
        break;
      case "o":
      case "O":
        project.setEnd(project.currentTime);
        break;
      case "ArrowLeft":
        project.step(e.shiftKey ? -(project.info.fps || 30) : -1);
        break;
      case "ArrowRight":
        project.step(e.shiftKey ? project.info.fps || 30 : 1);
        break;
      default:
        return;
    }
    e.preventDefault();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="app">
  <header>
    <div class="brand">
      <span class="logo">GIF</span>
      <span>GIF Maker</span>
      {#if project.info}
        <span class="file" title={project.info.path}>{project.info.fileName}</span>
      {/if}
    </div>
    <div class="actions">
      {#if project.info}
        <button class="ghost" onclick={openDialog}>{t("app.open")}</button>
      {/if}
      <div class="seg" role="group" aria-label="Language">
        <button class:on={i18n.locale === "ja"} onclick={() => (i18n.locale = "ja")}>日本語</button>
        <button class:on={i18n.locale === "en"} onclick={() => (i18n.locale = "en")}>EN</button>
      </div>
    </div>
  </header>

  {#if project.info}
    <main>
      <section class="stage">
        <Preview />
        <Timeline />
      </section>
      <SettingsPanel />
    </main>
    <ExportBar />
  {:else}
    <DropZone onpick={openDialog} active={dragOver} loading={project.loading} />
  {/if}

  {#if project.error}
    <div class="toast error" role="alert">
      <strong>{t("error.title")}</strong>
      <span>{project.error}</span>
      <button class="ghost" aria-label="close" onclick={() => (project.error = null)}>✕</button>
    </div>
  {/if}

  {#if dragOver && project.info}
    <div class="drop-overlay">{t("drop.release")}</div>
  {/if}
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100vh;
    position: relative;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    gap: 12px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-weight: 600;
    min-width: 0;
  }
  .logo {
    background: linear-gradient(135deg, #7c5cff, #ff5c8a);
    color: #fff;
    font-size: 11px;
    font-weight: 800;
    padding: 3px 6px;
    border-radius: 6px;
  }
  .file {
    color: var(--muted);
    font-weight: 400;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  main {
    display: grid;
    grid-template-columns: 1fr 300px;
    min-height: 0;
  }
  .stage {
    display: grid;
    grid-template-rows: 1fr auto;
    min-height: 0;
    min-width: 0;
  }
  .toast {
    position: absolute;
    left: 50%;
    bottom: 90px;
    transform: translateX(-50%);
    display: flex;
    gap: 10px;
    align-items: center;
    max-width: 70%;
    padding: 10px 14px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--danger);
    box-shadow: var(--shadow);
    z-index: 20;
  }
  .toast strong {
    color: var(--danger);
  }
  .drop-overlay {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    font-size: 22px;
    font-weight: 600;
    color: var(--accent);
    background: color-mix(in srgb, var(--bg) 80%, transparent);
    border: 3px dashed var(--accent);
    z-index: 30;
    pointer-events: none;
  }
</style>
