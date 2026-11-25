import type { ChatInputMode } from '@/context/workspace/workspaceContextTypes';

/** Free chat routes only visible @mentions; plan mode can explicitly select its lead. */
export function resolveUserMessageMentions(
  text: string,
  mode: ChatInputMode,
  workflowTargets?: string[],
): string[] {
  const explicit = Array.from(
    new Set(Array.from(text.matchAll(/(^|[^\p{L}\p{N}_.-])@([\p{L}\p{N}_-]+)/gu), match => match[2])),
  );
  return mode === 'workflow' ? (workflowTargets ?? explicit) : explicit;
}
