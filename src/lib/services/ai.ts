import { Channel, invoke } from '@tauri-apps/api/core';

export type Conversation = {
  id: string;
  title: string;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
};

export type ChatMessage = {
  id: string;
  conversationId: string;
  role: 'user' | 'assistant' | 'system' | string;
  content: string;
  model?: string | null;
  tokensIn?: number | null;
  tokensOut?: number | null;
  costUsd?: number | null;
  status: string;
  createdAt: string;
};

export type DailyUsage = {
  tokensIn: number;
  tokensOut: number;
  costUsd: number;
  date: string;
};

export type AiSettings = {
  provider: string;
  baseUrl: string;
  model: string;
  titleModel: string;
  temperature: number;
  dailySpendLimitUsd: number;
  hasApiKey: boolean;
  keyRequired: boolean;
};

export type StreamEvent =
  | {
      type: 'started';
      generationId: string;
      conversationId: string;
      userMessage: ChatMessage;
      assistantMessageId: string;
    }
  | { type: 'token'; text: string }
  | { type: 'done'; message: ChatMessage; usage: DailyUsage }
  | { type: 'cancelled'; message: ChatMessage }
  | { type: 'error'; message: string };

export function getAiSettings(): Promise<AiSettings> {
  return invoke<AiSettings>('get_ai_settings');
}

export function saveAiSettings(settings: {
  provider: string;
  baseUrl: string;
  model: string;
  titleModel: string;
  temperature: number;
  dailySpendLimitUsd: number;
}): Promise<AiSettings> {
  return invoke<AiSettings>('save_ai_settings', { settings });
}

export function saveApiKey(apiKey: string): Promise<AiSettings> {
  return invoke<AiSettings>('save_api_key', { apiKey });
}

export function validateApiKey(apiKey?: string): Promise<void> {
  return invoke<void>('validate_api_key', { apiKey: apiKey || null });
}

export function getDailyUsage(): Promise<DailyUsage> {
  return invoke<DailyUsage>('get_daily_usage');
}

export function listConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>('list_conversations');
}

export function createConversation(): Promise<Conversation> {
  return invoke<Conversation>('create_conversation');
}

export function archiveConversation(conversationId: string): Promise<void> {
  return invoke<void>('archive_conversation', { conversationId });
}

export function listMessages(conversationId: string): Promise<ChatMessage[]> {
  return invoke<ChatMessage[]>('list_messages', { conversationId });
}

export function cancelGeneration(generationId: string): Promise<void> {
  return invoke<void>('cancel_generation', { generationId });
}

export function sendMessage(
  conversationId: string | null,
  content: string,
  onEvent: (event: StreamEvent) => void,
): Promise<void> {
  const channel = new Channel<StreamEvent>();
  channel.onmessage = onEvent;
  return invoke<void>('send_message', { conversationId, content, onEvent: channel });
}
