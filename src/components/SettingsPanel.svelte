<script lang="ts">
  import { t, type Key } from "../lib/i18n.svelte";
  import { PRESETS, project, type PresetId } from "../lib/project.svelte";

  const presetIds = Object.keys(PRESETS) as PresetId[];
  const SPEEDS = [0.5, 1, 1.5, 2];

  const active = $derived(project.activePreset());
  const out = $derived(project.outSize);
  const info = $derived(project.info);

  function clampInt(v: number, lo: number, hi: number) {
    return Math.min(hi, Math.max(lo, Math.round(Number.isFinite(v) ? v : lo)));
  }
</script>

<aside class="panel">
  <h2>{t("settings.title")}</h2>

  <section>
    <div class="label">{t("settings.preset")}</div>
    <div class="presets">
      {#each presetIds as id (id)}
        <button class:on={active === id} onclick={() => project.applyPreset(id)}>
          <span class="name">{t(`preset.${id}` as Key)}</span>
          <span class="desc mono">{Math.min(PRESETS[id].width, project.maxWidth)}px<br />{Math.min(PRESETS[id].fps, project.maxFps)}fps</span>
        </button>
      {/each}
    </div>
  </section>

  <section>
    <div class="row">
      <label class="label" for="w">{t("settings.width")}</label>
      <span class="value"><input
          id="w"
          type="number"
          class="mono"
          min="64"
          max={project.maxWidth}
          value={project.settings.width}
          onchange={(e) => (project.settings.width = clampInt(e.currentTarget.valueAsNumber, 64, project.maxWidth))}
        /> px</span>
    </div>
    <input type="range" min="64" max={project.maxWidth} step="2" bind:value={project.settings.width} />
    <div class="sub">{t("settings.outSize", { w: out[0], h: out[1] })}</div>
  </section>

  <section>
    <div class="row">
      <label class="label" for="fps">{t("settings.fps")}</label>
      <span class="value mono">{project.settings.fps} fps</span>
    </div>
    <input id="fps" type="range" min="1" max={project.maxFps} step="1" bind:value={project.settings.fps} />
  </section>

  <section>
    <div class="row">
      <label class="label" for="q">{t("settings.quality")}</label>
      <span class="value mono">{project.settings.quality}</span>
    </div>
    <input id="q" type="range" min="1" max="100" step="1" bind:value={project.settings.quality} />
    <div class="sub">{t("settings.qualityHint")}</div>
  </section>

  <section class="inline">
    <span class="label">{t("settings.loop")}</span>
    <div class="seg">
      <button class:on={project.settings.loopForever} onclick={() => (project.settings.loopForever = true)}>{t("loop.forever")}</button>
      <button class:on={!project.settings.loopForever} onclick={() => (project.settings.loopForever = false)}>{t("loop.once")}</button>
    </div>
  </section>

  <section class="inline">
    <span class="label">{t("settings.speed")}</span>
    <div class="seg">
      {#each SPEEDS as s (s)}
        <button class:on={project.settings.speed === s} onclick={() => (project.settings.speed = s)}>{s}x</button>
      {/each}
    </div>
  </section>

  {#if info}
    <p class="source">
      {t("settings.source", { w: info.width, h: info.height, fps: Math.round(info.fps * 100) / 100, codec: info.codec })}
    </p>
  {/if}
  <p class="about">
    GIF Maker v{__APP_VERSION__} · AGPL-3.0 · github.com/togucchi/gif-maker<br />
    Uses gifski (AGPL-3.0) and FFmpeg (GPL, ffmpeg.org)
  </p>
</aside>

<style>
  .panel {
    border-left: 1px solid var(--border);
    background: var(--surface);
    padding: 16px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
  }
  section {
    display: grid;
    gap: 6px;
  }
  section.inline {
    grid-template-columns: 1fr auto;
    align-items: center;
  }
  .label {
    font-size: 13px;
    font-weight: 600;
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .value {
    font-size: 13px;
    color: var(--muted);
  }
  .value input {
    width: 70px;
    text-align: right;
  }
  .sub {
    font-size: 12px;
    color: var(--muted);
  }
  .presets {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }
  .presets button {
    display: grid;
    gap: 2px;
    padding: 8px 4px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--bg);
  }
  .presets button.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .presets .name {
    font-weight: 600;
    font-size: 13px;
  }
  .presets .desc {
    font-size: 10px;
    color: var(--muted);
  }
  input[type="range"] {
    width: 100%;
    accent-color: var(--accent);
  }
  .about {
    margin: 0;
    font-size: 10px;
    line-height: 1.5;
    color: var(--muted);
    opacity: 0.8;
    user-select: text;
  }
  .source {
    margin-top: auto;
    font-size: 11px;
    color: var(--muted);
  }
</style>
