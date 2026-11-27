import { useCallback, useEffect, useState } from 'react';
import { Brain, Check, Copy, MessagesSquare, RefreshCw, Users } from 'lucide-react';
import { agentRuntimeApi } from '@/lib/api';
import { copyPlainText } from '@/lib/clipboard';
import { useWorkspace } from '@/context/WorkspaceContext';
import { AgentCharacterAvatar } from './AgentCharacterAvatar';
import { runtimeDisplayName } from './AddAgentSheet';

type Runtime = { runner: string; ready: boolean };

const INSTALL: Array<{ runner: string; command: string }> = [
  { runner: 'CLAUDE_CODE', command: 'npm install -g @anthropic-ai/claude-code' },
  { runner: 'CODEX', command: 'npm install -g @openai/codex' },
  { runner: 'GEMINI', command: 'npm install -g @google/gemini-cli' },
];

/**
 * First screen when there is no group yet: two steps that tick themselves
 * off (agents found on this Mac, first group), and a preview of what to say.
 */
export function WelcomeGuide({ onCreateGroup }: { onCreateGroup: () => void }) {
  const { locale, showToast } = useWorkspace();
  const zh = locale === 'zh';
  const [runtimes, setRuntimes] = useState<Runtime[] | null>(null);
  const [checking, setChecking] = useState(false);

  const detect = useCallback(async (refresh: boolean) => {
    setChecking(true);
    try {
      if (refresh) await agentRuntimeApi.refreshLight().catch(() => undefined);
      const result = await agentRuntimeApi.list();
      setRuntimes(result.runners
        .filter(status => status.installed)
        .map(status => ({ runner: status.runner_type, ready: status.executable })));
    } catch {
      setRuntimes([]);
    } finally {
      setChecking(false);
    }
  }, []);
  useEffect(() => { void detect(false); }, [detect]);

  const ready = (runtimes ?? []).filter(item => item.ready);
  const agentsDone = ready.length > 0;
  const missing = INSTALL.filter(item => !(runtimes ?? []).some(found => found.runner === item.runner));

  const copy = (text: string) => void copyPlainText(text).then(ok => showToast(ok ? (zh ? '已复制，粘贴到终端运行' : 'Copied. Paste it into Terminal.') : (zh ? '复制失败' : 'Could not copy')));

  return (
    <div className="ui-welcome">
      <header className="ui-welcome-head">
        <img src="/logos/onebrain-logo.png" alt="" className="ui-welcome-logo" />
        <h1>{zh ? '把你的编程 Agent 拉进一个群' : 'Put your coding agents in one group chat'}</h1>
        <p>
          {zh
            ? 'Claude Code、Codex、Gemini 在你的 Mac 上一起聊、一起干活，共用一个大脑。用的是你自己的订阅，OneBrain 不碰你的密钥。'
            : 'Claude Code, Codex and Gemini talk and work together on your Mac and share one brain. They use your own subscriptions; OneBrain never sees your keys.'}
        </p>
      </header>

      <ol className="ui-welcome-steps">
        <li className="ui-panel ui-welcome-step" data-done={agentsDone || undefined}>
          <span className="ui-welcome-num">{agentsDone ? <Check size={16} /> : '1'}</span>
          <div className="ui-welcome-body">
            <h2><Users size={16} />{zh ? '找到你的 Agent' : 'Find your agents'}</h2>
            {runtimes === null ? (
              <p>{zh ? '正在查找这台 Mac 上装好的编程 Agent……' : 'Looking for coding agents on this Mac…'}</p>
            ) : (
              <>
                <p>
                  {agentsDone
                    ? (zh ? `找到 ${ready.length} 个可以直接用的：` : `Found ${ready.length} ready to use:`)
                    : (zh ? '还没找到能用的编程 Agent。装一个、在终端里登录一次，再点「重新检测」。' : 'No coding agent is ready yet. Install one, sign in once in Terminal, then check again.')}
                </p>
                {(runtimes ?? []).length > 0 && (
                  <div className="ui-welcome-agents">
                    {(runtimes ?? []).map(item => (
                      <span key={item.runner} className="ui-welcome-agent" data-ready={item.ready || undefined}>
                        <span className="paper-person-avatar"><AgentCharacterAvatar identity={item.runner}>{runtimeDisplayName(item.runner)[0]}</AgentCharacterAvatar></span>
                        <span>{runtimeDisplayName(item.runner)}<small>{item.ready ? (zh ? '可以用' : 'ready') : (zh ? '需要先登录' : 'sign in first')}</small></span>
                      </span>
                    ))}
                  </div>
                )}
                {missing.length > 0 && (
                  <details className="ui-welcome-install" open={!agentsDone}>
                    <summary>{zh ? '还想加别的？一行命令安装' : 'Add another with one command'}</summary>
                    {missing.map(item => (
                      <button key={item.runner} type="button" className="ui-welcome-cmd" onClick={() => copy(item.command)}>
                        <span>{runtimeDisplayName(item.runner)}</span><code>{item.command}</code><Copy size={13} />
                      </button>
                    ))}
                  </details>
                )}
                <button type="button" className="ui-btn" disabled={checking} onClick={() => void detect(true)}>
                  <RefreshCw size={14} />{checking ? (zh ? '检测中…' : 'Checking…') : (zh ? '重新检测' : 'Check again')}
                </button>
              </>
            )}
          </div>
        </li>

        <li className="ui-panel ui-welcome-step" data-locked={!agentsDone || undefined}>
          <span className="ui-welcome-num">2</span>
          <div className="ui-welcome-body">
            <h2><MessagesSquare size={16} />{zh ? '建第一个群' : 'Create your first group'}</h2>
            <p>
              {zh
                ? '聊天群用来讨论和分工；项目群绑定一个代码文件夹，Agent 在里面改代码，每次改动都能一键撤销。'
                : 'A chat group is for discussing and splitting work; a project group is tied to a code folder, where agents edit code and every run can be undone in one click.'}
            </p>
            <button type="button" className="ui-btn ui-btn-primary" disabled={!agentsDone} onClick={onCreateGroup}>
              {zh ? '建群' : 'Create group'}
            </button>
          </div>
        </li>

        <li className="ui-panel ui-welcome-step" data-locked>
          <span className="ui-welcome-num">3</span>
          <div className="ui-welcome-body">
            <h2><Brain size={16} />{zh ? '然后这样说话' : 'Then talk like this'}</h2>
            <ul className="ui-welcome-tips">
              <li><code>@codex</code>{zh ? ' 叫一个人干活，只有它会回复' : ' asks one agent; only it replies'}</li>
              <li><code>@所有人</code>{zh ? ' 每个人各说一句，适合听意见' : ' asks everyone once, for opinions'}</li>
              <li>{zh ? '不 @ 任何人：自动交给最合适的成员' : 'Mention nobody: the best-suited member answers'}</li>
              <li>{zh ? '输入框左下角切到「工作」：先出计划，你确认后再分步执行' : 'Switch to Work in the composer: a plan first, then steps after you approve'}</li>
            </ul>
          </div>
        </li>
      </ol>
    </div>
  );
}

/** Example messages for an empty chat; clicking one fills the composer. */
export function EmptyChatExamples({ sessionId, memberNames, zh, onPick }: {
  sessionId: string;
  memberNames: string[];
  zh: boolean;
  onPick: (detail: { sessionId: string; text: string; mode: 'free' | 'workflow' }) => void;
}) {
  const first = memberNames[0] ?? 'claude';
  const examples: Array<{ text: string; mode: 'free' | 'workflow'; note: string }> = [
    { text: zh ? `@${first} 看看这个项目的结构，告诉我从哪里开始` : `@${first} look at this project's structure and tell me where to start`, mode: 'free', note: zh ? '叫一个人' : 'Ask one agent' },
    { text: zh ? '@所有人 这个功能怎么做比较好？每人说一个思路' : '@all how should we build this? One idea each', mode: 'free', note: zh ? '问所有人' : 'Ask everyone' },
    { text: zh ? '给登录页加上"记住我"，写好测试' : 'Add "remember me" to the login page, with tests', mode: 'workflow', note: zh ? '工作模式：先出计划' : 'Work mode: plan first' },
  ];
  return (
    <div className="ui-empty-chat">
      <p className="ui-empty-chat-title">{zh ? '试试这样开始' : 'Try one of these'}</p>
      {examples.map(example => (
        <button key={example.text} type="button" className="ui-panel ui-empty-chat-example" onClick={() => onPick({ sessionId, text: example.text, mode: example.mode })}>
          <small>{example.note}</small>
          <span>{example.text}</span>
        </button>
      ))}
    </div>
  );
}
