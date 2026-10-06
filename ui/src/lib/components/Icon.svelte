<script lang="ts">
  // Schlichte Strich-Icons, damit keine Icon-Bibliothek nötig ist.
  type Name = 'plus' | 'folder' | 'settings' | 'close' | 'stop' | 'check' | 'alert' | 'retry' | 'edit' | 'chevron' | 'arrow-up' | 'arrow-down';
  let { name }: { name: Name } = $props();
</script>

<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  {#if name === 'plus'}
    <path d="M12 5v14M5 12h14" />
  {:else if name === 'folder'}
    <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
  {:else if name === 'settings'}
    <path d="M4 7h10M18 7h2M4 17h4M12 17h8" /><circle cx="16" cy="7" r="2" /><circle cx="10" cy="17" r="2" />
  {:else if name === 'close'}
    <path d="M6 6l12 12M18 6L6 18" />
  {:else if name === 'stop'}
    <rect x="7" y="7" width="10" height="10" rx="1.5" />
  {:else if name === 'check'}
    <path d="M5 12.5l4.5 4.5L19 7.5" />
  {:else if name === 'alert'}
    <path d="M12 8v5M12 16.5v.5" /><circle cx="12" cy="12" r="9" />
  {:else if name === 'edit'}
    <path d="M4 20h4L19 9l-4-4L4 16z" /><path d="M13.5 6.5l4 4" />
  {:else if name === 'arrow-up'}
    <path d="M12 19V5M6 11l6-6 6 6" />
  {:else if name === 'arrow-down'}
    <path d="M12 5v14M6 13l6 6 6-6" />
  {:else if name === 'chevron'}
    <path d="M7 10l5 5 5-5" />
  {:else if name === 'retry'}
    <path d="M4 12a8 8 0 1 0 2.5-5.8M4 4v4h4" />
  {/if}
</svg>
