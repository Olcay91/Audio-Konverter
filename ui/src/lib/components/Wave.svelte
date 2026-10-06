<script lang="ts">
  // Pegelbalken als Motiv der App. Bewegung zeigt Zustände an:
  // pulse = Dateien werden über das Fenster gezogen, wave = Dateien werden eingelesen.
  let {
    motion = 'none',
    size = 'large',
  }: { motion?: 'none' | 'pulse' | 'wave'; size?: 'large' | 'small' } = $props();

  // Höhen bewusst unregelmäßig wie ein echtes Signal.
  const LARGE = [0.35, 0.6, 0.85, 0.5, 0.7, 1, 0.65, 0.4, 0.75, 0.55, 0.3];
  const SMALL = [0.45, 0.8, 1, 0.6, 0.35];
  const bars = $derived(size === 'small' ? SMALL : LARGE);

  /**
   * Negative Verzögerung: Jeder Balken startet mitten in seiner Bewegung, versetzt
   * zum Nachbarn. So läuft sofort eine Welle von links nach rechts durch.
   */
  const delay = (i: number) => (motion === 'wave' ? `${-(bars.length - i) * 90}ms` : `${i * 70}ms`);
</script>

<span
  class="bars"
  class:large={size === 'large'}
  class:small={size === 'small'}
  class:pulse={motion === 'pulse'}
  class:wave={motion === 'wave'}
  aria-hidden="true"
>
  {#each bars as h, i}
    <span style:height="{h * 100}%" style:animation-delay={delay(i)}></span>
  {/each}
</span>

<style>
  .bars {
    display: inline-flex;
    align-items: center;
    flex: none;
  }
  .bars span {
    border-radius: 3px;
    background: var(--accent);
    opacity: 0.85;
    transform-origin: center;
  }
  .large {
    gap: 5px;
    height: 64px;
  }
  .large span {
    width: 6px;
  }
  .small {
    gap: 2px;
    height: 14px;
  }
  .small span {
    width: 3px;
    border-radius: 2px;
  }

  .pulse span {
    animation: pulse 0.9s ease-in-out infinite alternate;
  }
  .wave span {
    animation: wave 1s ease-in-out infinite;
  }
  @keyframes pulse {
    to {
      transform: scaleY(0.45);
    }
  }
  @keyframes wave {
    0%,
    100% {
      transform: scaleY(0.3);
    }
    50% {
      transform: scaleY(1);
    }
  }
</style>
