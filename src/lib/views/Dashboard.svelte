<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import type { IconName } from '../components/icons';
  import { hideWindow, setWindowState } from '../services/window';
  import Chat from './Chat.svelte';
  import Settings from './Settings.svelte';

  type NavItem = {
    id: string;
    label: string;
    icon: IconName;
    stage: number;
  };

  const nav: NavItem[] = [
    { id: 'assistant', label: 'Asistente', icon: 'chat', stage: 2 },
    { id: 'search', label: 'Buscar', icon: 'search', stage: 4 },
    { id: 'mail', label: 'Correo', icon: 'mail', stage: 5 },
    { id: 'calendar', label: 'Agenda', icon: 'calendar', stage: 5 },
    { id: 'habits', label: 'Hábitos', icon: 'habits', stage: 6 },
    { id: 'memory', label: 'Memoria', icon: 'memory', stage: 3 },
    { id: 'settings', label: 'Ajustes', icon: 'settings', stage: 8 },
  ];

  type StageStatus = 'done' | 'active' | 'pending';

  let section = $state<'assistant' | 'settings'>('assistant');

  const roadmap: { n: number; title: string; status: StageStatus }[] = [
    { n: 0, title: 'Cimientos', status: 'done' },
    { n: 1, title: 'Shell y widget', status: 'done' },
    { n: 2, title: 'Núcleo IA y chat', status: 'active' },
    { n: 3, title: 'Memoria', status: 'pending' },
    { n: 4, title: 'Buscador web', status: 'pending' },
    { n: 5, title: 'Gmail y Calendar', status: 'pending' },
    { n: 6, title: 'Recordatorios y hábitos', status: 'pending' },
    { n: 7, title: 'Herramientas y permisos', status: 'pending' },
    { n: 8, title: 'Voz y entrega', status: 'pending' },
  ];

  const completed = roadmap.filter((item) => item.status === 'done').length;
  const progress = Math.round((completed / roadmap.length) * 100);
</script>

<div class="shell">
  <aside class="sidebar">
    <div class="brand" data-tauri-drag-region>
      <span class="mark" aria-hidden="true"></span>
      <div data-tauri-drag-region>
        <strong data-tauri-drag-region>DazNova</strong>
        <small data-tauri-drag-region>Asistente personal</small>
      </div>
    </div>

    <nav>
      {#each nav as item (item.id)}
        <button
          class="nav-item"
          class:active={item.id === section}
          disabled={item.id !== 'assistant' && item.id !== 'settings'}
          onclick={() => {
            if (item.id === 'assistant' || item.id === 'settings') section = item.id;
          }}
        >
          <Icon name={item.icon} />
          <span>{item.label}</span>
          {#if item.id !== 'assistant' && item.id !== 'settings'}
            <em>E{item.stage}</em>
          {/if}
        </button>
      {/each}
    </nav>

    <div class="sidebar-footer">
      <div class="progress">
        <div class="progress-head">
          <span>Progreso</span>
          <span>{completed}/9</span>
        </div>
        <div class="track"><div class="fill" style="width: {progress}%"></div></div>
      </div>
      <button class="nav-item" onclick={() => setWindowState('expanded')}>
        <Icon name="collapse" />
        <span>Volver al panel</span>
      </button>
    </div>
  </aside>

  <main class="main">
    <header class="topbar" data-tauri-drag-region>
      <div data-tauri-drag-region>
        <h1 data-tauri-drag-region>{section === 'settings' ? 'Ajustes' : 'Asistente'}</h1>
        <p data-tauri-drag-region>Etapa 2 · núcleo de IA y chat</p>
      </div>
      <div class="actions">
        <button class="ghost" onclick={() => setWindowState('widget')} title="Minimizar a widget" aria-label="Minimizar a widget">
          <Icon name="collapse" />
        </button>
        <button class="ghost" onclick={() => hideWindow()} title="Ocultar" aria-label="Ocultar asistente">
          <Icon name="close" />
        </button>
      </div>
    </header>

    <div class="scroll" class:chat={section === 'assistant'}>
      {#if section === 'settings'}
        <Settings />
      {:else}
        <div class="chat-wrap">
          <Chat onOpenSettings={() => (section = 'settings')} />
        </div>
      {/if}
    </div>
  </main>
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: 232px 1fr;
    height: 100%;
    border-radius: var(--radius-lg);
    border: 1px solid var(--glass-border);
    background:
      radial-gradient(90% 60% at 100% 0%, rgba(123, 97, 255, 0.12), transparent 60%),
      rgba(8, 11, 18, 0.94);
    backdrop-filter: blur(var(--glass-blur)) saturate(140%);
    -webkit-backdrop-filter: blur(var(--glass-blur)) saturate(140%);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    padding: var(--space-4);
    border-right: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.015);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-2) var(--space-2) var(--space-1);
    cursor: grab;
    user-select: none;
  }

  .mark {
    width: 30px;
    height: 30px;
    flex: none;
    border-radius: var(--radius-full);
    background: var(--accent-gradient);
    box-shadow: var(--accent-glow);
  }

  .brand strong {
    display: block;
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
  }

  .brand small {
    display: block;
    font-size: var(--text-2xs);
    color: var(--text-muted);
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    text-align: left;
    transition:
      background var(--dur-fast) var(--ease-out),
      color var(--dur-fast) var(--ease-out);
  }

  .nav-item:hover:not(:disabled) {
    background: var(--glass-bg);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: linear-gradient(90deg, rgba(76, 201, 240, 0.14), rgba(123, 97, 255, 0.08));
    color: var(--text-primary);
    box-shadow: inset 0 0 0 1px rgba(76, 201, 240, 0.18);
  }

  .nav-item:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .nav-item em {
    margin-left: auto;
    font-style: normal;
    font-size: var(--text-2xs);
    color: var(--text-muted);
  }

  .sidebar-footer {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding-top: var(--space-3);
    border-top: 1px solid var(--glass-border);
  }

  .progress-head {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-2xs);
    color: var(--text-muted);
  }

  .track {
    height: 4px;
    margin-top: var(--space-2);
    border-radius: var(--radius-full);
    background: rgba(255, 255, 255, 0.06);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    border-radius: var(--radius-full);
    background: var(--accent-gradient);
    transition: width var(--dur-slow) var(--ease-out);
  }

  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--glass-border);
    user-select: none;
  }

  h1 {
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
    letter-spacing: -0.01em;
  }

  .topbar p {
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .actions {
    display: flex;
    gap: var(--space-1);
  }

  .ghost {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-xs);
    color: var(--text-secondary);
  }

  .ghost:hover {
    background: var(--glass-bg-strong);
    color: var(--text-primary);
  }

  .scroll {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
    min-height: 0;
  }

  .scroll.chat {
    overflow: hidden;
    padding: var(--space-4) var(--space-6) var(--space-6);
  }

  .chat-wrap {
    flex: 1;
    min-height: 0;
  }
</style>
