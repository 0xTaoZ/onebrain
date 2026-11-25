import type { CreateChatMessageRequest } from '@/types';

/** Derive once from the existing optimistic message id; retries retain that id. */
export function withChatOpKey(data: CreateChatMessageRequest): CreateChatMessageRequest {
  if (data.opKey) return data;
  const meta = data.meta;
  const key = meta && typeof meta === 'object' && !Array.isArray(meta) ? meta.client_message_id : undefined;
  return typeof key === 'string' && /^[A-Za-z0-9_-]{1,64}$/.test(key)
    ? { ...data, opKey: key }
    : data;
}

export interface NeedsAckDetails { active_count: number; items: Array<{id: string; title: string}>; }
export class NeedsAckError extends Error {
  constructor(public readonly pending: NeedsAckDetails) { super('needs_ack'); this.name = 'NeedsAckError'; }
}
export interface DispatchAcknowledgement {
  pending: NeedsAckDetails;
  retry: () => Promise<void>;
  busy?: boolean;
  error?: string;
}
