<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { t } from "../lib/i18n.svelte";
  import { LONG_RANGE_WARN, project } from "../lib/project.svelte";
  import { api } from "../lib/api";
  import { formatBytes, formatDuration } from "../lib/format";

  const est = $derived(project.estimate);
  const enc = $derived(project.encoding);

  async function saveAs() {
    const r = project.result;
    if (!r || !project.info) return;
    const stem = project.info.fileName.replace(/\.[^.]+$/, "");
    const dest = await save({
      defaultPath: `${stem}.gif`,
      filters: [{ name: "GIF", extensions: ["gif"] }],
    });
    if (!dest) return;
    try {
      await api.saveGif(r.path, dest);
      project.notice = t("export.saved", { path: dest });
    } catch (e) {
      project.error = String(e);
    }
  }
</script>

<footer>
  <div class="info">
    {#if enc}
      <div class="progress">
        <span>{t("export.creating", { done: enc.done, total: enc.total || "…" })}</span>
        <progress value={enc.done} max={enc.total || 1}></progress>
      </div>
    {:else if project.notice}
      <span class="ok">✓ {project.notice}</span>
    {:else}
      <span class="size" class:dim={project.estimating}>
        {#if est}
          {t("export.estimate", { size: formatBytes(est.bytes) })}
        {:else}
          {t("export.estimating")}
        {/if}
      </span>
      <span class="meta mono">
        {project.outSize[0]}×{project.outSize[1]} · {project.settings.fps}fps ·
        {t("export.frames", { n: est?.frames ?? "–" })} · {formatDuration(project.rangeLength / project.settings.speed)}
      </span>
      {#if project.rangeLength > LONG_RANGE_WARN}
        <span class="warn">⚠ {t("export.longWarn")}</span>
      {/if}
    {/if}
  </div>

  <div class="buttons">
    {#if enc}
      <button class="ghost" onclick={() => project.cancelEncode()}>{t("export.cancel")}</button>
    {:else}
      {#if project.result}
        <span class="result">{t("export.result", { size: formatBytes(project.result.bytes) })}</span>
      {/if}
      <button class:primary={!project.result || project.resultStale} onclick={() => project.encode()} disabled={project.rangeLength <= 0}>
        {project.result ? t("export.recreate") : t("export.create")}
      </button>
      <button class:primary={!!project.result && !project.resultStale} disabled={!project.result} onclick={saveAs}>
        {t("export.save")}
      </button>
    {/if}
  </div>
</footer>

<style>
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    min-height: 60px;
  }
  .info {
    display: flex;
    align-items: baseline;
    gap: 14px;
    min-width: 0;
    flex-wrap: wrap;
  }
  .size {
    font-size: 18px;
    font-weight: 700;
    transition: opacity 0.2s;
  }
  .size.dim {
    opacity: 0.5;
  }
  .meta {
    font-size: 12px;
    color: var(--muted);
  }
  .warn {
    font-size: 12px;
    color: var(--warn);
  }
  .ok {
    color: var(--ok);
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .progress {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
  }
  .progress progress {
    width: 280px;
  }
  .buttons {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
  }
  .buttons button {
    padding: 9px 20px;
    font-size: 14px;
  }
  .result {
    font-size: 13px;
    color: var(--muted);
  }
</style>
