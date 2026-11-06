<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import Widget from './lib/views/Widget.svelte';
  import ExpandedPanel from './lib/views/ExpandedPanel.svelte';
  import Dashboard from './lib/views/Dashboard.svelte';
  import { getWindowState, type WindowState } from './lib/services/window';
  import { chat } from './lib/stores/chat.svelte';

  let view = $state<WindowState>('widget');

  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listen<WindowState>('window-state-changed', (event) => {
      view = event.payload;
    }).then((stop) => {
      if (disposed) {
        stop();
      } else {
        unlisten = stop;
      }
    });

    // Estado real del backend al arrancar: evita desajustes si la ventana
    // cambió por la bandeja antes de que el frontend estuviera listo.
    void getWindowState()
      .then((state) => {
        if (!disposed) view = state;
      })
      .catch(() => {});

    void chat.hydrate().catch(() => {});

    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

{#if view === 'widget'}
  <Widget />
{:else if view === 'expanded'}
  <ExpandedPanel />
{:else}
  <Dashboard />
{/if}
