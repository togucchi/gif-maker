<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { project } from "../lib/project.svelte";
  import { formatBytes, formatTime } from "../lib/format";

  let raf = 0;

  // While playing, follow the video every frame and wrap inside the selection.
  function tick() {
    const v = project.video;
    if (!v) return;
    project.currentTime = v.currentTime;
    if (project.loopRange && v.currentTime >= project.end) {
      v.currentTime = project.start;
      project.currentTime = project.start;
    }
    if (!v.paused) raf = requestAnimationFrame(tick);
  }

  function onPlay() {
    project.playing = true;
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(tick);
  }

  function onPause() {
    project.playing = false;
    cancelAnimationFrame(raf);
    if (project.video) project.currentTime = project.video.currentTime;
  }

  function onLoaded() {
    const v = project.video;
    // Some codecs "load" but decode no picture (e.g. HEVC without the OS extension).
    if (v && v.videoWidth === 0) project.useProxy();
    else if (v) v.currentTime = project.currentTime;
  }

  $effect(() => () => cancelAnimationFrame(raf));
</script>

<div class="preview">
  <div class="tabs" role="tablist">
    <button role="tab" class:on={project.tab === "video"} aria-selected={project.tab === "video"} onclick={() => (project.tab = "video")}>
      {t("tab.video")}
    </button>
    <button role="tab" class:on={project.tab === "gif"} aria-selected={project.tab === "gif"} onclick={() => (project.tab = "gif")}>
      {t("tab.gif")}
      {#if project.result}<span class="badge">{formatBytes(project.result.bytes)}</span>{/if}
    </button>
  </div>

  <div class="screen" class:hidden={project.tab !== "video"}>
    <!-- svelte-ignore a11y_media_has_caption -->
    <video
      bind:this={project.video}
      src={project.videoSrc}
      preload="auto"
      muted
      onplay={onPlay}
      onpause={onPause}
      onended={onPause}
      onloadedmetadata={onLoaded}
      onerror={() => project.useProxy()}
      onclick={() => project.togglePlay()}
    ></video>
    {#if project.proxyProgress !== null}
      <div class="overlay">
        <p>{t("proxy.making", { p: Math.round(project.proxyProgress * 100) })}</p>
        <progress value={project.proxyProgress} max="1"></progress>
      </div>
    {/if}
  </div>

  <div class="screen checker" class:hidden={project.tab !== "gif"}>
    {#if project.result}
      <img src={project.result.url} alt="GIF preview" />
      {#if project.resultStale}
        <div class="stale">{t("gif.stale")}</div>
      {/if}
    {:else}
      <p class="empty">{t("gif.empty")}</p>
    {/if}
  </div>

  {#if project.tab === "video"}
    <div class="controls">
      <button class="play" onclick={() => project.togglePlay()} title={project.playing ? t("player.pause") : t("player.play")}>
        {#if project.playing}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="5" width="4" height="14" /><rect x="14" y="5" width="4" height="14" /></svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M7 5v14l12-7z" /></svg>
        {/if}
      </button>
      <span class="time mono">
        {formatTime(project.currentTime)} <span class="muted">/ {formatTime(project.info?.duration ?? 0)}</span>
      </span>
      <label class="check">
        <input type="checkbox" bind:checked={project.loopRange} />
        {t("player.loopRange")}
      </label>
    </div>
  {/if}
</div>

<style>
  .preview {
    display: grid;
    grid-template-rows: auto 1fr auto;
    min-height: 0;
    padding: 12px 16px 0;
    gap: 8px;
  }
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tabs button {
    padding: 6px 14px;
    border-radius: 8px;
    background: transparent;
    border: 1px solid transparent;
    color: var(--muted);
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .tabs button.on {
    background: var(--surface);
    border-color: var(--border);
    color: var(--text);
    font-weight: 600;
  }
  .badge {
    font-size: 11px;
    padding: 1px 6px;
    border-radius: 99px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .screen {
    position: relative;
    min-height: 0;
    display: grid;
    place-items: center;
    background: #000;
    border-radius: 12px;
    overflow: hidden;
  }
  .screen.hidden {
    display: none;
  }
  .screen.checker {
    background-color: var(--checker-a);
    background-image: conic-gradient(var(--checker-b) 25%, transparent 0 50%, var(--checker-b) 0 75%, transparent 0);
    background-size: 20px 20px;
  }
  video,
  img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    display: block;
  }
  video {
    width: 100%;
    height: 100%;
    cursor: pointer;
  }
  .overlay {
    position: absolute;
    inset: 0;
    display: grid;
    place-content: center;
    gap: 10px;
    text-align: center;
    color: #fff;
    background: rgba(0, 0, 0, 0.7);
    padding: 24px;
  }
  .overlay progress {
    width: 320px;
    justify-self: center;
  }
  .stale {
    position: absolute;
    top: 10px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--warn-bg);
    color: var(--warn);
    padding: 6px 12px;
    border-radius: 8px;
    font-size: 13px;
  }
  .empty {
    color: var(--muted);
    background: var(--surface);
    padding: 12px 16px;
    border-radius: 10px;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 14px;
    padding-bottom: 4px;
  }
  .play {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    padding: 0;
    background: var(--accent);
    color: #fff;
    border: none;
  }
  .time {
    font-size: 14px;
  }
  .check {
    margin-left: auto;
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 13px;
    color: var(--muted);
  }
</style>
