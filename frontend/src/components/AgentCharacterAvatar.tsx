import type { ReactNode } from 'react';

/** Each runtime family has one pixel character; unknown identities keep their letter mark. */
const CHARACTERS: Array<[RegExp, string]> = [
  [/deepseek|deep_seek|dsh/, 'deepseek'],
  [/claude|anthropic|sonnet|opus|haiku/, 'claude'],
  [/codex|openai|gpt-/, 'codex'],
  [/antigravity|gemini|google/, 'antigravity'],
  [/qwen|tongyi|千问/, 'qwen'],
  [/workbuddy|codebuddy|tencent|腾讯/, 'workbuddy'],
  [/opencode|\bamp\b|openteams|kimi|copilot|cursor/, 'wizard'],
];

export function agentCharacterFor(identity?: string | null): string | null {
  const value = identity?.toLowerCase() ?? '';
  return CHARACTERS.find(([pattern]) => pattern.test(value))?.[1] ?? null;
}

export function AgentCharacterAvatar({ identity, children }: { identity?: string | null; children: ReactNode }) {
  const character = agentCharacterFor(identity);
  if (!character) return <>{children}</>;
  return (
    <span className="agent-character-avatar" aria-hidden="true">
      <span className="agent-character-fallback">{children}</span>
      <img className="agent-character-image" src={`/avatars/${character}-pixel.png`} alt="" loading="lazy" draggable={false} />
    </span>
  );
}
