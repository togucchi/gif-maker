<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { LONG_RANGE_WARN, project } from "../lib/project.svelte";
  import { formatDuration, formatTime, parseTime } from "../lib/format";

  type Mode = "start" | "end" | "move" | "seek";

  let track: HTMLDivElement | undefined = $state();
  let trackW = $state(1);
  let viewStart = $state(0);
  let viewEnd = $state(1);
  let drag: { mode: Mode; offset: number } | null = null;

  const duration = $derived(project.info?.duration ?? 0);
  const span = $derived(Math.max(0.01, viewEnd - viewStart));
  const zoomed = $derived(span < duration - 0.001);

  // Reset the view for each new file. Long recordings start zoomed to the
  // first minute so a few-second selection is actually grabbable.
  const INITIAL_VIEW = 60;
  $effect(() => {
    void project.info?.path;
    const d = project.info?.duration ?? 1;
    viewStart = 0;
    viewEnd = d > INITIAL_VIEW * 2 ? INITIAL_VIEW : d;
  });

  // ---- overview strip (whole video, drag to pan) ----
  let overview: HTMLDivElement | undefined = $state();
  let panning = false;
  const opct = (time: number) => (duration ? (time / duration) * 100 : 0);

  function panTo(e: PointerEvent) {
    if (!overview) return;
    const r = overview.getBoundingClientRect();
    const center = ((e.clientX - r.left) / r.width) * duration;
    setView(center - span / 2, span);
  }

  // Keep the playhead visible during playback when zoomed in.
  $effect(() => {
    const ct = project.currentTime;
    if (!project.playing || drag) return;
    const s = viewEnd - viewStart;
    if (ct < viewStart || ct > viewEnd) setView(ct - s * 0.1, s);
  });

  const pct = (time: number) => ((time - viewStart) / span) * 100;

  function setView(start: number, s: number) {
    s = Math.min(Math.max(s, Math.min(1, duration)), duration);
    start = Math.min(Math.max(0, start), duration - s);
    viewStart = start;
    viewEnd = start + s;
  }

  function timeAt(clientX: number) {
    if (!track) return 0;
    const r = track.getBoundingClientRect();
    const time = viewStart + ((clientX - r.left) / r.width) * span;
    return Math.min(Math.max(0, time), duration);
  }

  function down(e: PointerEvent, mode: Mode) {
    if (e.button !== 0 || !track) return;
    e.stopPropagation();
    e.preventDefault();
    track.setPointerCapture(e.pointerId);
    project.video?.pause();
    drag = { mode, offset: timeAt(e.clientX) - project.start };
    apply(e);
  }

  function apply(e: PointerEvent) {
    if (!drag) return;
    const time = timeAt(e.clientX);
    switch (drag.mode) {
      case "start":
        project.setStart(time);
        project.seek(project.start);
        break;
      case "end":
        project.setEnd(time);
        project.seek(project.end);
        break;
      case "move":
        project.moveRange(time - drag.offset);
        project.seek(project.start);
        break;
      case "seek":
        project.seek(time);
        break;
    }
  }

  function up() {
    drag = null;
  }

  $effect(() => {
    const el = track;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const horizontal = e.shiftKey || Math.abs(e.deltaX) > Math.abs(e.deltaY);
      if (horizontal) {
        const delta = e.shiftKey ? e.deltaY || e.deltaX : e.deltaX;
        setView(viewStart + (delta / trackW) * span, span);
      } else {
        const anchor = timeAt(e.clientX);
        const next = Math.min(Math.max(span * Math.exp(e.deltaY * 0.0015), Math.min(1, duration)), duration);
        setView(anchor - (anchor - viewStart) * (next / span), next);
      }
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  });

  const STEPS = [0.1, 0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 900, 1800, 3600];
  const ticks = $derived.by(() => {
    const maxTicks = Math.max(2, trackW / 90);
    const step = STEPS.find((s) => span / s <= maxTicks) ?? 7200;
    const out: { time: number; label: string }[] = [];
    for (let x = Math.ceil(viewStart / step) * step; x <= viewEnd + 1e-6; x += step) {
      const full = formatTime(x);
      out.push({ time: x, label: step >= 1 ? full.slice(0, -3) : full });
    }
    return out;
  });

  const tiles = $derived.by(() => {
    const n = project.thumbs.length;
    if (!n || !duration) return [];
    const each = duration / n;
    return project.thumbs
      .map((url, i) => ({ url, a: i * each, b: (i + 1) * each }))
      .filter((x) => x.b >= viewStart && x.a <= viewEnd);
  });

  function commitTime(e: Event, which: "start" | "end") {
    const input = e.currentTarget as HTMLInputElement;
    const v = parseTime(input.value);
    if (v !== null) {
      if (which === "start") project.setStart(v);
      else project.setEnd(v);
      project.seek(which === "start" ? project.start : project.end);
    }
    input.value = formatTime(which === "start" ? project.start : project.end);
  }

  function nudge(e: KeyboardEvent, which: "start" | "end") {
    if (e.key !== "ArrowUp" && e.key !== "ArrowDown") return;
    e.preventDefault();
    const d = (e.key === "ArrowUp" ? 1 : -1) * (e.shiftKey ? 1 : 0.1);
    if (which === "start") project.setStart(project.start + d);
    else project.setEnd(project.end + d);
    project.seek(which === "start" ? project.start : project.end);
  }
</script>

<div class="timeline">
  {#if duration > INITIAL_VIEW * 2}
    <div
      class="overview"
      bind:this={overview}
      role="scrollbar"
      aria-controls="timeline-track"
      aria-valuenow={viewStart}
      tabindex="-1"
      onpointerdown={(e) => {
        panning = true;
        overview?.setPointerCapture(e.pointerId);
        panTo(e);
      }}
      onpointermove={(e) => panning && panTo(e)}
      onpointerup={() => (panning = false)}
    >
      <div class="o-sel" style:left="{opct(project.start)}%" style:width="max(2px, {opct(project.end) - opct(project.start)}%)"></div>
      <div class="o-view" style:left="{opct(viewStart)}%" style:width="{opct(viewEnd) - opct(viewStart)}%"></div>
      <div class="o-head" style:left="{opct(project.currentTime)}%"></div>
    </div>
  {/if}

  <div class="ruler">
    {#each ticks as tick (tick.time)}
      <span class="tick mono" class:edge={pct(tick.time) < 3} style:left="{pct(tick.time)}%">{tick.label}</span>
    {/each}
  </div>

  <div
    id="timeline-track"
    class="track"
    bind:this={track}
    bind:clientWidth={trackW}
    role="slider"
    tabindex="-1"
    aria-valuemin={0}
    aria-valuemax={duration}
    aria-valuenow={project.currentTime}
    onpointerdown={(e) => down(e, "seek")}
    onpointermove={apply}
    onpointerup={up}
    onpointercancel={up}
  >
    <div class="thumbs">
      {#each tiles as tile (tile.url)}
        <div
          class="tile"
          style:left="{pct(tile.a)}%"
          style:width="{((tile.b - tile.a) / span) * 100}%"
          style:background-image="url('{tile.url}')"
        ></div>
      {/each}
    </div>

    <div class="shade" style:left="0" style:width="{Math.max(0, pct(project.start))}%"></div>
    <div class="shade" style:left="{Math.min(100, pct(project.end))}%" style:right="0"></div>

    <div
      class="selection"
      style:left="{pct(project.start)}%"
      style:width="{pct(project.end) - pct(project.start)}%"
      role="presentation"
      onpointerdown={(e) => down(e, "move")}
    >
      <div class="handle left" role="presentation" onpointerdown={(e) => down(e, "start")}></div>
      <div class="handle right" role="presentation" onpointerdown={(e) => down(e, "end")}></div>
    </div>

    <div class="playhead" style:left="{pct(project.currentTime)}%"></div>
  </div>

  <div class="fields">
    <label class="field">
      <span>{t("timeline.start")}</span>
      <input
        class="mono"
        value={formatTime(project.start)}
        onchange={(e) => commitTime(e, "start")}
        onkeydown={(e) => nudge(e, "start")}
      />
      <button class="ghost small" title={t("timeline.setStart")} onclick={() => project.setStart(project.currentTime)}>[ I</button>
    </label>

    <div class="length">
      {t("timeline.length")} <strong class="mono">{formatDuration(project.rangeLength)}</strong>
      {#if project.rangeLength > LONG_RANGE_WARN}
        <span class="warn" title={t("export.longWarn")}>⚠</span>
      {/if}
    </div>

    <label class="field">
      <span>{t("timeline.end")}</span>
      <input
        class="mono"
        value={formatTime(project.end)}
        onchange={(e) => commitTime(e, "end")}
        onkeydown={(e) => nudge(e, "end")}
      />
      <button class="ghost small" title={t("timeline.setEnd")} onclick={() => project.setEnd(project.currentTime)}>O ]</button>
    </label>

    <div class="view">
      <span class="hint">{t("timeline.hint")}</span>
      <button class="ghost small" disabled={!zoomed} onclick={() => setView(0, duration)}>{t("timeline.fit")}</button>
    </div>
  </div>
</div>

<style>
  .timeline {
    padding: 8px 16px 14px;
    display: grid;
    gap: 4px;
    user-select: none;
  }
  .ruler {
    position: relative;
    height: 16px;
    overflow: hidden;
  }
  .tick {
    position: absolute;
    font-size: 10px;
    color: var(--muted);
    transform: translateX(-50%);
    white-space: nowrap;
  }
  .tick.edge {
    transform: none;
  }
  .tick.edge::after {
    left: 0;
  }
  .overview {
    position: relative;
    height: 12px;
    border-radius: 6px;
    background: var(--surface-2);
    cursor: pointer;
    touch-action: none;
    margin-bottom: 4px;
  }
  .o-sel,
  .o-view,
  .o-head {
    position: absolute;
    top: 0;
    bottom: 0;
    pointer-events: none;
  }
  .o-sel {
    background: var(--accent);
    border-radius: 2px;
  }
  .o-view {
    border: 1.5px solid var(--text);
    border-radius: 6px;
    opacity: 0.6;
    min-width: 6px;
  }
  .o-head {
    width: 2px;
    margin-left: -1px;
    background: var(--text);
  }
  .tick::after {
    content: "";
    position: absolute;
    left: 50%;
    bottom: -4px;
    height: 4px;
    border-left: 1px solid var(--border-strong);
  }
  .track {
    position: relative;
    height: 64px;
    border-radius: 8px;
    background: var(--surface-2);
    overflow: hidden;
    cursor: pointer;
    touch-action: none;
  }
  .thumbs,
  .tile {
    position: absolute;
    top: 0;
    bottom: 0;
  }
  .thumbs {
    left: 0;
    right: 0;
  }
  .tile {
    background-size: auto 100%;
    background-repeat: repeat-x;
    background-position: left center;
  }
  .shade {
    position: absolute;
    top: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.55);
    pointer-events: none;
  }
  .selection {
    position: absolute;
    top: 0;
    bottom: 0;
    border: 3px solid var(--accent);
    border-left-width: 0;
    border-right-width: 0;
    box-sizing: border-box;
    cursor: grab;
  }
  .handle {
    position: absolute;
    top: -3px;
    bottom: -3px;
    width: 14px;
    background: var(--accent);
    cursor: ew-resize;
  }
  .handle::after {
    content: "";
    position: absolute;
    top: 50%;
    left: 50%;
    width: 2px;
    height: 18px;
    transform: translate(-50%, -50%);
    background: rgba(255, 255, 255, 0.9);
    border-radius: 1px;
  }
  .handle.left {
    left: 0;
    border-radius: 6px 0 0 6px;
  }
  .handle.right {
    right: 0;
    border-radius: 0 6px 6px 0;
  }
  .playhead {
    position: absolute;
    top: -2px;
    bottom: -2px;
    width: 2px;
    margin-left: -1px;
    background: #fff;
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.5);
    pointer-events: none;
  }
  .fields {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: 6px;
    flex-wrap: wrap;
  }
  .field {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--muted);
  }
  .field input {
    width: 92px;
    text-align: center;
  }
  .length {
    font-size: 13px;
    color: var(--muted);
  }
  .length strong {
    color: var(--text);
  }
  .warn {
    color: var(--warn);
    cursor: help;
  }
  .view {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .hint {
    font-size: 11px;
    color: var(--muted);
  }
</style>
