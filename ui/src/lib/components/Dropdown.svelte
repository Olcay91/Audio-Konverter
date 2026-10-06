<script lang="ts" module>
  export interface Option<V> {
    value: V;
    label: string;
    /** Zweite, graue Zeile im Menü */
    description?: string;
    /** Überschrift; wird angezeigt, wenn sie sich gegenüber dem vorigen Eintrag ändert */
    group?: string;
  }
</script>

<script lang="ts" generics="T">
  // Auswahlliste im Stil des Preset-Dropdowns, als Ersatz für <select>, dessen
  // aufgeklapptes Menü sich nicht gestalten lässt.
  //
  // Das Menü wird an <body> gehängt und fest positioniert. So wird es weder von
  // scrollenden Bereichen (Einstellungs-Panel) noch von Dialogen mit `transform`
  // (Preset-Editor) abgeschnitten. Es öffnet nach unten, wenn dort Platz ist, sonst nach oben.
  // Der Fokus bleibt immer auf dem Button; die Tastatur steuert die Markierung im Menü.
  import { tick } from 'svelte';
  import Icon from './Icon.svelte';

  let {
    value,
    options,
    onchange,
    label,
    size = 'md',
    block = false,
  }: {
    value: T;
    options: Option<T>[];
    onchange: (value: T) => void;
    /** Für Screenreader (aria-label) */
    label: string;
    /** md = 34 px (Standard), sm = 30 px (Einstellungen), xs = 28 px (Listenkopf) */
    size?: 'md' | 'sm' | 'xs';
    /** Volle Breite statt nach Inhalt */
    block?: boolean;
  } = $props();

  const uid = `dd-${Math.random().toString(36).slice(2, 9)}`;

  let open = $state(false);
  /** Per Tastatur markierter Eintrag, solange das Menü offen ist */
  let active = $state(0);
  let trigger = $state<HTMLButtonElement>();
  let menu = $state<HTMLDivElement>();
  let position = $state('');

  const selectedIndex = $derived(options.findIndex((o) => o.value === value));
  const current = $derived(options[selectedIndex]);

  /** Hängt das Menü an <body>, damit es nicht abgeschnitten wird. */
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }

  function place() {
    if (!trigger) return;
    const r = trigger.getBoundingClientRect();
    const margin = 8;
    const below = window.innerHeight - r.bottom - margin;
    const above = r.top - margin;
    const wanted = Math.min(360, (menu?.scrollHeight ?? 320) + 2);
    const up = below < wanted && above > below;
    const maxHeight = Math.max(120, Math.min(360, up ? above - 6 : below - 6));
    const left = Math.max(margin, Math.min(r.left, window.innerWidth - Math.max(r.width, 220) - margin));
    position =
      `left: ${left}px; min-width: ${r.width}px; max-height: ${maxHeight}px; ` +
      (up ? `bottom: ${window.innerHeight - r.top + 6}px;` : `top: ${r.bottom + 6}px;`);
  }

  async function show() {
    active = Math.max(0, selectedIndex);
    open = true;
    place();
    await tick();
    place(); // jetzt mit echter Menühöhe
    scrollActiveIntoView();
  }

  function hide() {
    open = false;
  }

  function choose(index: number) {
    const option = options[index];
    if (option && option.value !== value) onchange(option.value);
    hide();
  }

  async function scrollActiveIntoView() {
    await tick();
    menu?.querySelector(`#${uid}-${active}`)?.scrollIntoView({ block: 'nearest' });
  }

  function onKeydown(e: KeyboardEvent) {
    const step = e.key === 'ArrowDown' ? 1 : e.key === 'ArrowUp' ? -1 : 0;
    if (!open) {
      // Wie bei einer normalen Auswahlliste: Pfeiltasten wechseln den Wert direkt.
      if (step) {
        e.preventDefault();
        const next = options[selectedIndex + step];
        if (next) onchange(next.value);
      }
      return;
    }
    if (step) {
      e.preventDefault();
      active = Math.min(options.length - 1, Math.max(0, active + step));
      scrollActiveIntoView();
    } else if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      choose(active);
    } else if (e.key === 'Escape') {
      // Nur das Menü schließen, nicht zusätzlich Dialog oder Einstellungs-Panel.
      e.preventDefault();
      e.stopPropagation();
      hide();
    } else if (e.key === 'Tab') {
      hide();
    }
  }

  function onWindowPointer(e: MouseEvent) {
    if (!open) return;
    const target = e.target as Node;
    if (trigger?.contains(target) || menu?.contains(target)) return;
    hide();
  }

  // Beim Scrollen außerhalb des Menüs oder bei Größenänderung schließen statt mitzuwandern.
  function onWindowScroll(e: Event) {
    if (open && !(menu && menu.contains(e.target as Node))) hide();
  }
</script>

<svelte:window onmousedown={onWindowPointer} onresize={hide} />
<svelte:document onscrollcapture={onWindowScroll} />

<button
  bind:this={trigger}
  type="button"
  class="select trigger"
  class:sm={size === 'sm'}
  class:xs={size === 'xs'}
  class:block
  aria-haspopup="listbox"
  aria-expanded={open}
  aria-controls={open ? `${uid}-menu` : undefined}
  aria-activedescendant={open ? `${uid}-${active}` : undefined}
  aria-label={label}
  title={current?.description ?? ''}
  onclick={() => (open ? hide() : show())}
  onkeydown={onKeydown}
>
  <span class="current">{current?.label ?? ''}</span>
  <Icon name="chevron" />
</button>

{#if open}
  <!-- mousedown verhindert, dass der Button den Fokus verliert -->
  <div
    bind:this={menu}
    use:portal
    id="{uid}-menu"
    class="menu"
    role="listbox"
    aria-label={label}
    style={position}
    onmousedown={(e) => e.preventDefault()}
  >
    {#each options as option, i (i)}
      {#if option.group && option.group !== options[i - 1]?.group}
        <div class="group">{option.group}</div>
      {/if}
      <div
        id="{uid}-{i}"
        class="item"
        class:selected={i === selectedIndex}
        class:active={i === active}
        role="option"
        aria-selected={i === selectedIndex}
        tabindex="-1"
        onclick={() => choose(i)}
        onkeydown={() => {}}
        onmouseenter={() => (active = i)}
      >
        <span class="name">{option.label}</span>
        {#if option.description}<span class="summary">{option.description}</span>{/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  /* Gleiche Optik wie Preset- und Format-Dropdown */
  .trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-width: 120px;
    max-width: 100%;
    text-align: left;
    cursor: pointer;
  }
  .trigger.block {
    width: 100%;
  }
  .trigger.sm {
    height: 30px;
  }
  .trigger.xs {
    height: 28px;
    min-width: 0;
    padding: 0 8px;
    gap: 6px;
  }
  .trigger :global(svg) {
    width: 16px;
    height: 16px;
    flex: none;
    color: var(--muted);
  }
  .trigger.xs :global(svg) {
    width: 14px;
    height: 14px;
  }
  .current {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .menu {
    position: fixed;
    z-index: 60;
    max-width: calc(100vw - 16px);
    overflow-y: auto;
    padding: 6px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    box-shadow: 0 12px 32px rgb(0 0 0 / 0.16);
  }
  .group {
    padding: 8px 10px 4px;
    color: var(--muted);
    font-size: 0.82em;
  }
  .item {
    display: grid;
    gap: 1px;
    padding: 7px 10px;
    border-radius: var(--radius);
    cursor: pointer;
    white-space: nowrap;
  }
  .item.active {
    background: var(--accent-soft);
  }
  .item.selected {
    background: var(--accent-soft);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .selected .name {
    font-weight: 600;
  }
  .name,
  .summary {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .summary {
    color: var(--muted);
    font-size: 0.86em;
  }
</style>
