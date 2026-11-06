<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import { hideWindow, setWindowState } from '../services/window';
  import Chat from './Chat.svelte';
</script>

<section class="panel">
  <header class="bar" data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <span class="pulse" aria-hidden="true"></span>
      <span class="name" data-tauri-drag-region>DazNova</span>
    </div>
    <div class="actions">
      <button
        class="ghost"
        onclick={() => setWindowState('dashboard')}
        title="Panel completo"
        aria-label="Abrir panel completo"
      >
        <Icon name="expand" />
      </button>
      <button class="ghost" onclick={() => hideWindow()} title="Ocultar" aria-label="Ocultar asistente">
        <Icon name="close" />
      </button>
    </div>
  </header>

  <div class="content">
    <Chat onOpenSettings={() => setWindowState('dashboard')} />
  </div>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    border-radius: var(--radius-lg);
    border: 1px solid var(--glass-border);
    background:
      radial-gradient(120% 60% at 15% 0%, rgba(123, 97, 255, 0.16), transparent 60%),
      radial-gradient(100% 50% at 90% 100%, rgba(76, 201, 240, 0.1), transparent 55%),
      rgba(9, 12, 19, 0.86);
    backdrop-filter: blur(var(--glass-blur)) saturate(140%);
    -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(140%);
    box-shadow: var(--shadow-lg), var(--shadow-inset);
    overflow: hidden;
  }

  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-3) var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--glass-border);
    user-select: none;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    cursor: grab;
  }

  .pulse {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--accent-cyan);
    box-shadow: 0 0 10px var(--accent-cyan);
    animation: breathe 3.4s var(--ease-in-out) infinite;
  }

  .name {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    letter-spacing: 0.02em;
  }

  .actions {
    display: flex;
    gap: var(--space-1);
  }

  .ghost {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-xs);
    color: var(--text-secondary);
    transition:
      background var(--dur-fast) var(--ease-out),
      color var(--dur-fast) var(--ease-out);
  }

  .ghost:hover:not(:disabled) {
    background: var(--glass-bg-strong);
    color: var(--text-primary);
  }

  .content {
    flex: 1;
    min-height: 0;
    padding: var(--space-3) var(--space-4) var(--space-4);
  }

  @keyframes breathe {
    50% {
      opacity: 0.35;
      box-shadow: 0 0 4px var(--accent-cyan);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .pulse {
      animation: none;
    }
  }
</style>
