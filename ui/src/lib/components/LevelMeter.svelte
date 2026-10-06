<script lang="ts">
  // Fortschritt als Pegelanzeige aus Segmenten – das wiedererkennbare Element der App.
  let { value, phase = 'running' }: { value: number; phase?: 'running' | 'done' | 'waiting' } = $props();
  const pct = $derived(Math.round(Math.min(1, Math.max(0, value)) * 100));
</script>

<div
  class="meter"
  class:done={phase === 'done'}
  class:waiting={phase === 'waiting'}
  class:indeterminate={phase === 'running' && value === 0}
  role="progressbar"
  aria-valuemin="0"
  aria-valuemax="100"
  aria-valuenow={pct}
>
  <span class="fill" style:width="{pct}%"></span>
</div>

<style>
  .meter {
    position: relative;
    height: 6px;
    border-radius: 2px;
    overflow: hidden;
    background: var(--surface-sunk);
    /* Segmentierung: 4px Balken, 2px Lücke */
    mask: repeating-linear-gradient(90deg, #000 0 4px, transparent 4px 6px);
    -webkit-mask: repeating-linear-gradient(90deg, #000 0 4px, transparent 4px 6px);
  }
  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--accent);
    transition: width 0.25s linear;
  }
  .done .fill {
    background: var(--ok);
  }
  .waiting .fill {
    width: 0 !important;
  }
  .indeterminate .fill {
    width: 30% !important;
    animation: sweep 1.4s ease-in-out infinite;
  }
  @keyframes sweep {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(340%);
    }
  }
</style>
