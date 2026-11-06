<script lang="ts">
  import { chat } from '../stores/chat.svelte';
  import { saveAiSettings, saveApiKey, validateApiKey } from '../services/ai';

  let provider = $state('openai');
  let baseUrl = $state('https://api.openai.com/v1');
  let model = $state('gpt-4o-mini');
  let titleModel = $state('gpt-4o-mini');
  let temperature = $state(0.7);
  let dailySpendLimitUsd = $state(1);
  let apiKey = $state('');
  let status = $state('');
  let busy = $state(false);

  $effect(() => {
    const settings = chat.settings;
    if (!settings) return;
    provider = settings.provider;
    baseUrl = settings.baseUrl;
    model = settings.model;
    titleModel = settings.titleModel;
    temperature = settings.temperature;
    dailySpendLimitUsd = settings.dailySpendLimitUsd;
  });

  function onProviderChange() {
    if (provider === 'openai') {
      baseUrl = 'https://api.openai.com/v1';
      if (model === 'llama3.2') model = 'gpt-4o-mini';
      if (titleModel === 'llama3.2') titleModel = 'gpt-4o-mini';
    }
    if (provider === 'ollama') {
      baseUrl = 'http://127.0.0.1:11434/v1';
      if (model.includes('gpt-')) model = 'llama3.2';
      if (titleModel.includes('gpt-')) titleModel = model;
    }
  }

  async function save() {
    busy = true;
    status = '';
    try {
      chat.settings = await saveAiSettings({
        provider,
        baseUrl,
        model,
        titleModel,
        temperature,
        dailySpendLimitUsd,
      });
      if (apiKey.trim()) {
        chat.settings = await saveApiKey(apiKey.trim());
        apiKey = '';
      }
      status = 'Ajustes guardados.';
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    } finally {
      busy = false;
    }
  }

  async function testConnection() {
    busy = true;
    status = '';
    try {
      await save();
      await validateApiKey(apiKey.trim() || undefined);
      status = 'Conexión correcta.';
    } catch (error) {
      status = error instanceof Error ? error.message : String(error);
    } finally {
      busy = false;
    }
  }

  const usage = $derived(chat.usage);
  const hasKey = $derived(Boolean(chat.settings?.hasApiKey));
</script>

<section class="settings">
  <header>
    <h2>Motor de IA</h2>
    <p>El mismo cliente sirve para OpenAI, Ollama y cualquier API compatible. La clave no se guarda en SQLite.</p>
  </header>

  {#if usage}
    <div class="usage">
      <div>
        <span>Gasto de hoy</span>
        <strong>${usage.costUsd.toFixed(4)}</strong>
      </div>
      <div>
        <span>Tokens</span>
        <strong>{usage.tokensIn + usage.tokensOut}</strong>
      </div>
      <div>
        <span>Límite</span>
        <strong>${dailySpendLimitUsd.toFixed(2)}</strong>
      </div>
    </div>
  {/if}

  <label>
    Proveedor
    <select bind:value={provider} onchange={onProviderChange}>
      <option value="openai">OpenAI</option>
      <option value="ollama">Ollama (local)</option>
      <option value="custom">Compatible (URL propia)</option>
    </select>
  </label>

  <label>
    URL base
    <input bind:value={baseUrl} spellcheck="false" />
  </label>

  <label>
    Modelo
    <input bind:value={model} spellcheck="false" />
  </label>

  <label>
    Modelo para títulos
    <input bind:value={titleModel} spellcheck="false" />
  </label>

  <label>
    Temperatura ({temperature.toFixed(1)})
    <input type="range" min="0" max="2" step="0.1" bind:value={temperature} />
  </label>

  <label>
    Límite diario (USD)
    <input type="number" min="0" step="0.05" bind:value={dailySpendLimitUsd} />
  </label>

  <label>
    API key {hasKey ? '(guardada en el llavero)' : ''}
    <input type="password" bind:value={apiKey} placeholder={hasKey ? '••••••••  (deja vacío para no cambiar)' : 'sk-…'} />
  </label>

  <div class="actions">
    <button class="primary" onclick={save} disabled={busy}>Guardar</button>
    <button class="secondary" onclick={testConnection} disabled={busy}>Probar conexión</button>
  </div>

  {#if status}
    <p class="status">{status}</p>
  {/if}
</section>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    max-width: 520px;
  }

  h2 {
    font-size: var(--text-lg);
    font-weight: var(--weight-semibold);
  }

  header p {
    margin-top: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .usage {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-3);
  }

  .usage div {
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.03);
  }

  .usage span {
    display: block;
    font-size: var(--text-2xs);
    color: var(--text-muted);
  }

  .usage strong {
    display: block;
    margin-top: var(--space-1);
    font-size: var(--text-md);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  input,
  select {
    height: 36px;
    padding: 0 var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--glass-border);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-primary);
  }

  input[type='range'] {
    height: 24px;
    padding: 0;
  }

  .actions {
    display: flex;
    gap: var(--space-2);
  }

  .primary,
  .secondary {
    height: 36px;
    padding: 0 var(--space-4);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
  }

  .primary {
    background: var(--accent-gradient);
    color: var(--text-inverse);
  }

  .secondary {
    border: 1px solid var(--glass-border);
    color: var(--text-primary);
  }

  .primary:disabled,
  .secondary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .status {
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }
</style>
