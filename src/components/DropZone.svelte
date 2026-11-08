<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  let { onpick, active, loading }: { onpick: () => void; active: boolean; loading: boolean } = $props();
</script>

<button class="zone" class:active onclick={onpick} disabled={loading}>
  <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
    <rect x="3" y="5" width="18" height="14" rx="2" />
    <path d="M10 9.5v5l4-2.5z" fill="currentColor" stroke="none" />
  </svg>
  {#if loading}
    <span class="title">{t("loading")}</span>
  {:else if active}
    <span class="title">{t("drop.release")}</span>
  {:else}
    <span class="title">{t("drop.title")}</span>
    <span class="sub">{t("drop.sub")}</span>
  {/if}
  <span class="steps">{t("drop.steps")}</span>
</button>

<style>
  .zone {
    grid-row: 2 / 4;
    margin: 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    border: 2px dashed var(--border-strong);
    border-radius: 18px;
    background: var(--surface);
    color: var(--muted);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .zone:hover,
  .zone.active {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-soft);
  }
  .title {
    font-size: 22px;
    font-weight: 600;
    color: var(--text);
  }
  .sub {
    font-size: 14px;
  }
  .steps {
    margin-top: 28px;
    font-size: 13px;
    color: var(--muted);
  }
</style>
