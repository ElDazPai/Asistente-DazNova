import {
  archiveConversation,
  cancelGeneration,
  createConversation,
  getAiSettings,
  getDailyUsage,
  listConversations,
  listMessages,
  sendMessage,
  type AiSettings,
  type ChatMessage,
  type Conversation,
  type DailyUsage,
  type StreamEvent,
} from '../services/ai';

class ChatStore {
  conversations = $state<Conversation[]>([]);
  messages = $state<ChatMessage[]>([]);
  activeId = $state<string | null>(null);
  settings = $state<AiSettings | null>(null);
  usage = $state<DailyUsage | null>(null);
  streaming = $state(false);
  generationId = $state<string | null>(null);
  error = $state<string | null>(null);
  draft = $state('');
  loaded = $state(false);

  async hydrate() {
    const [settings, usage, conversations] = await Promise.all([
      getAiSettings(),
      getDailyUsage(),
      listConversations(),
    ]);
    this.settings = settings;
    this.usage = usage;
    this.conversations = conversations;
    if (!this.activeId && conversations[0]) {
      await this.open(conversations[0].id);
    }
    this.loaded = true;
  }

  async refreshSettings() {
    this.settings = await getAiSettings();
    this.usage = await getDailyUsage();
  }

  async open(id: string) {
    this.activeId = id;
    this.messages = await listMessages(id);
    this.error = null;
  }

  async newConversation() {
    const created = await createConversation();
    this.conversations = [created, ...this.conversations];
    this.activeId = created.id;
    this.messages = [];
    this.error = null;
  }

  async removeActive() {
    if (!this.activeId) return;
    const id = this.activeId;
    await archiveConversation(id);
    this.conversations = this.conversations.filter((item) => item.id !== id);
    this.messages = [];
    this.activeId = this.conversations[0]?.id ?? null;
    if (this.activeId) {
      this.messages = await listMessages(this.activeId);
    }
  }

  async send() {
    const content = this.draft.trim();
    if (!content || this.streaming) return;

    this.draft = '';
    this.error = null;
    this.streaming = true;

    const optimistic: ChatMessage = {
      id: `local-${Date.now()}`,
      conversationId: this.activeId ?? '',
      role: 'user',
      content,
      status: 'complete',
      createdAt: new Date().toISOString(),
    };
    this.messages = [...this.messages, optimistic];

    try {
      await sendMessage(this.activeId, content, (event) => this.onEvent(event, optimistic.id));
    } catch (error) {
      this.error = error instanceof Error ? error.message : String(error);
      this.messages = this.messages.filter((message) => message.status !== 'streaming');
    } finally {
      this.streaming = false;
      this.generationId = null;
      this.conversations = await listConversations();
      this.usage = await getDailyUsage();
    }
  }

  async stop() {
    if (this.generationId) {
      await cancelGeneration(this.generationId);
    }
  }

  private onEvent(event: StreamEvent, optimisticId: string) {
    if (event.type === 'started') {
      this.generationId = event.generationId;
      this.activeId = event.conversationId;
      this.messages = [
        ...this.messages.filter((message) => message.id !== optimisticId),
        event.userMessage,
        {
          id: event.assistantMessageId,
          conversationId: event.conversationId,
          role: 'assistant',
          content: '',
          status: 'streaming',
          createdAt: new Date().toISOString(),
        },
      ];
      return;
    }

    if (event.type === 'token') {
      this.messages = this.messages.map((message) =>
        message.status === 'streaming'
          ? { ...message, content: message.content + event.text }
          : message,
      );
      return;
    }

    if (event.type === 'done') {
      this.messages = this.messages.map((message) =>
        message.id === event.message.id ? event.message : message,
      );
      this.usage = event.usage;
      return;
    }

    if (event.type === 'cancelled') {
      if (!event.message.content) {
        this.messages = this.messages.filter((message) => message.id !== event.message.id);
      } else {
        this.messages = this.messages.map((message) =>
          message.id === event.message.id ? event.message : message,
        );
      }
      return;
    }

    if (event.type === 'error') {
      this.error = event.message;
      this.messages = this.messages.filter((message) => message.status !== 'streaming');
    }
  }
}

export const chat = new ChatStore();
