<script lang="ts">
  import { renderMarkdown } from '../markdown';
  import type { ChatMessage } from '../services/ai';

  let { message }: { message: ChatMessage } = $props();

  const isUser = $derived(message.role === 'user');
  const html = $derived(isUser ? '' : renderMarkdown(message.content));
</script>

<article class="bubble" class:user={isUser} class:assistant={!isUser}>
  {#if isUser}
    <p>{message.content}</p>
  {:else if message.status === 'streaming' && !message.content}
    <p class="thinking">Pensando…</p>
  {:else}
    <div class="md">{@html html}</div>
    {#if message.status === 'cancelled'}
      <p class="meta">Respuesta cancelada</p>
    {/if}
    {#if message.status === 'complete' && message.tokensOut}
      <p class="meta">
        {message.tokensIn ?? 0} → {message.tokensOut} tokens
        {#if message.costUsd}
          · ${message.costUsd.toFixed(4)}
        {/if}
      </p>
    {/if}
  {/if}
</article>

<style>
  .bubble {
    max-width: 92%;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
  }

  .user {
    margin-left: auto;
    background: linear-gradient(135deg, rgba(76, 201, 240, 0.22), rgba(123, 97, 255, 0.18));
    border: 1px solid rgba(76, 201, 240, 0.22);
    white-space: pre-wrap;
  }

  .assistant {
    margin-right: auto;
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--glass-border);
  }

  .thinking {
    color: var(--text-muted);
    font-style: italic;
  }

  .meta {
    margin-top: var(--space-2);
    font-size: var(--text-2xs);
    color: var(--text-muted);
  }

  .md :global(p + p) {
    margin-top: var(--space-2);
  }

  .md :global(ul) {
    margin: var(--space-2) 0 0 var(--space-4);
  }

  .md :global(code) {
    font-family: var(--font-mono);
    font-size: 0.92em;
    padding: 0 4px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.35);
  }

  .md :global(pre) {
    margin-top: var(--space-2);
    padding: var(--space-3);
    overflow-x: auto;
    border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.4);
  }

  .md :global(pre code) {
    padding: 0;
    background: none;
  }
</style>
