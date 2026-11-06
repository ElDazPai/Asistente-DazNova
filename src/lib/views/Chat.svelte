<script lang="ts">
  import { tick } from 'svelte';
  import Icon from '../components/Icon.svelte';
  import MessageBubble from '../components/MessageBubble.svelte';
  import { chat } from '../stores/chat.svelte';

  let { onOpenSettings }: { onOpenSettings?: () => void } = $props();

  let scroller = $state<HTMLElement | null>(null);

  $effect(() => {
    chat.messages;
    chat.streaming;
    void tick().then(() => {
      if (scroller) scroller.scrollTop = scroller.scrollHeight;
    });
  });

  function onKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      void chat.send();
    }
  }

  const needsKey = $derived(
    Boolean(chat.settings?.keyRequired && !chat.settings.hasApiKey),
  );
</script>

<div class="chat">
  <div class="toolbar">
    <select
      class="thread"
      value={chat.activeId ?? ''}
      onchange={(event) => {
        const id = event.currentTarget.value;
        if (id) void chat.open(id);
      }}
      aria-label="Conversación"
    >
      {#if chat.conversations.length === 0}
        <option value="">Nueva conversación</option>
      {/if}
      {#each chat.conversations as item (item.id)}
        <option value={item.id}>{item.title}</option>
      {/each}
    </select>
    <button class="ghost" onclick={() => chat.newConversation()} title="Nueva conversación" aria-label="Nueva conversación">
      <Icon name="plus" />
    </button>
  </div>

  <div class="log" bind:this={scroller}>
    {#if chat.messages.length === 0}
      <div class="empty">
        <div class="halo" aria-hidden="true"></div>
        <h2>Hola, soy DazNova</h2>
        {#if needsKey}
          <p>Para conversar, guarda una API key en Ajustes.</p>
          {#if onOpenSettings}
            <button class="link" onclick={onOpenSettings}>Abrir ajustes</button>
          {/if}
        {:else}
          <p>Pregunta lo que necesites. Puedes cancelar una respuesta a mitad de camino.</p>
        {/if}
      </div>
    {:else}
      {#each chat.messages as message (message.id)}
        <MessageBubble {message} />
      {/each}
    {/if}
  </div>

  {#if chat.error}
    <p class="error" role="alert">{chat.error}</p>
  {/if}

  <form
    class="composer"
    onsubmit={(event) => {
      event.preventDefault();
      void chat.send();
    }}
  >
    <div class="field">
      <textarea
        bind:value={chat.draft}
        onkeydown={onKeyDown}
        placeholder={needsKey ? 'Configura la API key para escribir…' : 'Escribe un mensaje…'}
        aria-label="Mensaje"
        rows="1"
        disabled={needsKey}
      ></textarea>
      {#if chat.streaming}
        <button type="button" class="send stop" onclick={() => chat.stop()} title="Detener" aria-label="Detener generación">
          <Icon name="stop" />
        </button>
      {:else}
        <button type="submit" class="send" disabled={needsKey || !chat.draft.trim()} title="Enviar" aria-label="Enviar">
          <Icon name="send" />
        </button>
      {/if}
    </div>
    <p class="hint">Enter para enviar · Shift+Enter salto de línea · Escucha {chat.settings?.model ?? 'el modelo activo'}</p>
  </form>
</div>

<style>
  .chat {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
  }

  .toolbar {
    display: flex;
    gap: var(--space-2);
    padding: 0 0 var(--space-3);
  }

  .thread {
    flex: 1;
    min-width: 0;
    height: 32px;
    padding: 0 var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-primary);
  }

  .ghost {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-xs);
    color: var(--text-secondary);
  }

  .ghost:hover {
    background: var(--glass-bg-strong);
    color: var(--text-primary);
  }

  .log {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding-right: 2px;
  }

  .empty {
    position: relative;
    margin: auto;
    max-width: 32ch;
    text-align: center;
    padding: var(--space-6) 0;
  }

  .halo {
    position: absolute;
    top: -50px;
    left: 50%;
    width: 160px;
    height: 160px;
    transform: translateX(-50%);
    border-radius: var(--radius-full);
    background: var(--accent-gradient);
    filter: blur(58px);
    opacity: 0.2;
    pointer-events: none;
  }

  h2 {
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
  }

  .empty p {
    margin-top: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .link {
    margin-top: var(--space-3);
    color: var(--accent-cyan);
    font-size: var(--text-sm);
  }

  .error {
    margin: var(--space-2) 0;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--danger-soft);
    color: var(--danger);
    font-size: var(--text-xs);
  }

  .composer {
    padding-top: var(--space-3);
  }

  .field {
    display: flex;
    align-items: flex-end;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-1) var(--space-1) var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.03);
  }

  textarea {
    flex: 1;
    min-width: 0;
    max-height: 120px;
    resize: none;
    padding: var(--space-2) 0;
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
  }

  textarea:disabled {
    cursor: not-allowed;
  }

  .send {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    margin-bottom: 2px;
    border-radius: var(--radius-full);
    background: var(--accent-gradient);
    color: var(--text-inverse);
  }

  .send:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .send.stop {
    background: var(--danger);
    color: var(--text-primary);
  }

  .hint {
    margin-top: var(--space-2);
    text-align: center;
    font-size: var(--text-2xs);
    color: var(--text-muted);
  }
</style>
