<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  type BackendStatus = {
    version: string;
    os: string;
    arch: string;
  };

  let status = $state<BackendStatus | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    invoke<BackendStatus>('backend_status')
      .then((value) => {
        status = value;
      })
      .catch((reason) => {
        error = String(reason);
      });
  });
</script>

<main class="shell">
  <section class="card glass">
    <div class="orb" aria-hidden="true"></div>

    <header>
      <p class="eyebrow">Asistente personal</p>
      <h1>DazNova</h1>
    </header>

    <p class="tagline">
      Tu asistente inteligente, siempre disponible en el escritorio.
    </p>

    <footer class="status">
      {#if status}
        <span class="dot online" aria-hidden="true"></span>
        <span>
          Backend conectado · v{status.version} · {status.os}/{status.arch}
        </span>
      {:else if error}
        <span class="dot offline" aria-hidden="true"></span>
        <span>Sin conexión con el backend</span>
      {:else}
        <span class="dot pending" aria-hidden="true"></span>
        <span>Iniciando…</span>
      {/if}
    </footer>

    {#if error}
      <pre class="error">{error}</pre>
    {/if}
  </section>
</main>

<style>
  .shell {
    display: grid;
    place-items: center;
    height: 100%;
    padding: var(--space-8);
    background:
      radial-gradient(circle at 20% 0%, rgba(123, 97, 255, 0.18), transparent 55%),
      radial-gradient(circle at 85% 100%, rgba(76, 201, 240, 0.14), transparent 50%),
      var(--bg-void);
  }

  .card {
    position: relative;
    width: min(520px, 100%);
    padding: var(--space-10) var(--space-8) var(--space-6);
    border-radius: var(--radius-xl);
    overflow: hidden;
  }

  .orb {
    position: absolute;
    top: -60px;
    right: -40px;
    width: 200px;
    height: 200px;
    border-radius: var(--radius-full);
    background: var(--accent-gradient);
    filter: blur(60px);
    opacity: 0.35;
    pointer-events: none;
  }

  .eyebrow {
    font-size: var(--text-2xs);
    font-weight: var(--weight-semibold);
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  h1 {
    margin-top: var(--space-2);
    font-size: var(--text-2xl);
    font-weight: var(--weight-semibold);
    letter-spacing: -0.02em;
    line-height: var(--leading-tight);
    background: var(--accent-gradient);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }

  .tagline {
    margin-top: var(--space-4);
    max-width: 34ch;
    font-size: var(--text-lg);
    color: var(--text-secondary);
  }

  .status {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-10);
    padding-top: var(--space-4);
    border-top: 1px solid var(--glass-border);
    font-size: var(--text-xs);
    color: var(--text-muted);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: var(--radius-full);
    flex: none;
  }

  .online {
    background: var(--success);
    box-shadow: 0 0 10px var(--success);
  }

  .offline {
    background: var(--danger);
    box-shadow: 0 0 10px var(--danger);
  }

  .pending {
    background: var(--warning);
    animation: pulse 1.2s var(--ease-in-out) infinite;
  }

  .error {
    margin-top: var(--space-3);
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--danger-soft);
    color: var(--danger);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: pre-wrap;
    word-break: break-word;
  }

  @keyframes pulse {
    50% {
      opacity: 0.3;
    }
  }
</style>
