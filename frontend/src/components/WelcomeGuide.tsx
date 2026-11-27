import { useCallback, useEffect, useState } from 'react';
import { Brain, Check, Copy, MessagesSquare, RefreshCw, Users } from 'lucide-react';
import { agentRuntimeApi, chatMessagesApi } from '@/lib/api';
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

export function WelcomeGuide({ sessionId, onCreateGroup, onFinish }: {
  sessionId: string | null;
  onCreateGroup: (runners?: string[]) => void;
  onFinish: () => void;
}) {
  const { locale, showToast, members, membersAsync, refreshMessages } = useWorkspace();
  const zh = locale === 'zh';
  const [runtimes, setRuntimes] = useState<Runtime[] | null>(null);
  const [checking, setChecking] = useState(false);
  const [detectError, setDetectError] = useState(false);
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);
  const [sendError, setSendError] = useState('');

  const detect = useCallback(async (refresh: boolean) => {
    setChecking(true);
    setDetectError(false);
    try {
      if (refresh) await agentRuntimeApi.refreshLight().catch(() => undefined);
      const result = await agentRuntimeApi.list();
      setRuntimes(result.runners
        .filter(status => status.installed)
        .map(status => ({ runner: status.runner_type, ready: status.executable })));
    } catch {
      setRuntimes([]);
      setDetectError(true);
    } finally {
      setChecking(false);
    }
  }, []);
  useEffect(() => { void detect(false); }, [detect]);

  const ready = (runtimes ?? []).filter(item => item.ready);
  const agentsDone = sessionId !== null;
  const sendExample = async (example: Example) => {
    if (!sessionId || !members.length || membersAsync.loading || membersAsync.error || sending || sent) return;
    setSending(true);
    setSendError('');
    try {
      await chatMessagesApi.send(sessionId, {
        sender_type: 'user',
        sender_id: null,
        content: example.text,
        meta: { app_language: locale, chat_input_mode: example.mode },
      });
      setSent(true);
      void refreshMessages();
    } catch (error) {
      setSendError(error instanceof Error ? error.message : String(error));
    } finally {
      setSending(false);
    }
  };
  useEffect(() => {
    if (!sent) return;
    const timer = window.setTimeout(onFinish, 450);
    return () => window.clearTimeout(timer);
  }, [sent, onFinish]);
  const missing = INSTALL.filter(item => !(runtimes ?? []).some(found => found.runner === item.runner));

  const copy = (text: string) => void copyPlainText(text).then(ok => showToast(ok ? (zh ? '已复制，粘贴到终端运行' : 'Copied. Paste it into Terminal.') : (zh ? '复制失败' : 'Could not copy')));

  return (
    <div className="ui-welcome">
      <header className="ui-welcome-head">
        <img src="/logos/onebrain-logo.png" alt="" className="ui-welcome-logo" />
        <h1>{zh ? '把你的编程 Agent 拉进一个群' : 'Put your coding agents in one group chat'}</h1>
        <p>
          {zh
            ? '把本机的编程 Agent 放进同一个群，一起讨论、开发和审核。'
            : 'Bring your local coding agents into one chat to discuss, build, and review together.'}
        </p>
      </header>

      <ol className="ui-welcome-steps">
        <li className="ui-panel ui-welcome-step" data-done={agentsDone || undefined}>
          <span className="ui-welcome-num">{agentsDone ? <Check size={16} /> : '1'}</span>
          <div className="ui-welcome-body">
            <h2><Users size={16} />{zh ? '接入成员' : 'Add agents'}</h2>
            {agentsDone ? (
              <p>{zh ? '成员已加入。' : 'Your agents have joined.'}</p>
            ) : runtimes === null ? (
              <p>{zh ? '正在查找这台 Mac 上装好的编程 Agent……' : 'Looking for coding agents on this Mac…'}</p>
            ) : (
              <>
                <p>
                  {detectError
                    ? (zh ? '暂时无法检测本机 Agent，请重试。' : 'Could not detect local agents. Try again.')
                    : ready.length
                      ? (zh ? `找到 ${ready.length} 个 Agent。点「加入」选择群类型。` : `Found ${ready.length} agents. Click Add to choose a group type.`)
                      : (zh ? '请先安装并登录一个编程 Agent，再重新检测。' : 'Install and sign in to a coding agent, then check again.')}
                </p>
                {(runtimes ?? []).length > 0 && (
                  <div className="ui-welcome-agents">
                    {(runtimes ?? []).map(item => (
                      <span key={item.runner} className="ui-welcome-agent" data-ready={item.ready || undefined}>
                        <span className="paper-person-avatar"><AgentCharacterAvatar identity={item.runner}>{runtimeDisplayName(item.runner)[0]}</AgentCharacterAvatar></span>
                        <span>{runtimeDisplayName(item.runner)}</span>
                        {item.ready
                          ? <button type="button" className="ui-sheet-add" onClick={() => onCreateGroup([item.runner])}>{zh ? '加入' : 'Add'}</button>
                          : <small>{zh ? '暂不可用' : 'Unavailable'}</small>}
                      </span>
                    ))}
                  </div>
                )}
                {missing.length > 0 && (
                  <details className="ui-welcome-install" open={!ready.length && !detectError}>
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

        <li className="ui-panel ui-welcome-step" data-done={agentsDone || undefined}>
          <span className="ui-welcome-num">{agentsDone ? <Check size={16} /> : '2'}</span>
          <div className="ui-welcome-body">
            <h2><MessagesSquare size={16} />{zh ? '建一个群' : 'Create a group'}</h2>
            {agentsDone ? (
              <p>{zh ? '群聊已建好。' : 'Your group is ready.'}</p>
            ) : <>
              <p>{zh ? '聊天群用来讨论。项目群会绑定一个工作文件夹。' : 'Use a chat group to discuss, or link a working folder in a project group.'}</p>
              <button type="button" className="ui-btn ui-btn-primary" disabled={!ready.length} onClick={() => onCreateGroup()}>
                {zh ? '选择群类型' : 'Choose a group type'}
              </button>
            </>}
          </div>
        </li>

        <li className="ui-panel ui-welcome-step" data-locked={!agentsDone || undefined} data-done={sent || undefined}>
          <span className="ui-welcome-num">{sent ? <Check size={16} /> : '3'}</span>
          <div className="ui-welcome-body">
            <h2><Brain size={16} />{zh ? '发第一条消息' : 'Send your first message'}</h2>
            {agentsDone ? <>
              <p>{sent
                ? (zh ? '消息已发送。' : 'Message sent.')
                : (zh ? '点一个示例就会发送给对应成员。' : 'Click an example to send it to your agents.')}</p>
              {!sent && <>
                {examplesFor(members.map(member => member.name.replace(/^@/, '')), zh).map(example => (
                  <button key={example.note} type="button" className="ui-panel ui-empty-chat-example"
                    disabled={sending || membersAsync.loading || !!membersAsync.error || !members.length}
                    onClick={() => void sendExample(example)}>
                    <small>{example.note}</small><span>{example.text}</span>
                  </button>
                ))}
                {sendError && <p role="alert">{sendError}</p>}
                <button type="button" className="ui-btn" disabled={sending} onClick={onFinish}>
                  {zh ? '直接进入聊天' : 'Go to chat'}
                </button>
              </>}
            </> : <p>{zh ? '建群后可以试试点名一位成员、@所有人，或用工作模式制定计划。' : 'After creating a group, try asking one agent, everyone, or making a plan in Work mode.'}</p>}
          </div>
        </li>
      </ol>
    </div>
  );
}

type Example = { text: string; mode: 'free' | 'workflow'; note: string };

function examplesFor(memberNames: string[], zh: boolean): Example[] {
  const first = memberNames[0] ?? 'claude';
  return [
    { text: zh ? `@${first} 介绍一下你能帮我做什么` : `@${first} tell me what you can help with`, mode: 'free', note: zh ? '叫一个人' : 'Ask one agent' },
    { text: zh ? '@所有人 我们怎么分工？每人说一个建议' : '@all how should we split the work? One idea each', mode: 'free', note: zh ? '问所有人' : 'Ask everyone' },
    { text: zh ? '给小组项目拟一份开发和审核计划' : 'Draft a development and review plan for a team project', mode: 'workflow', note: zh ? '工作模式：先出计划' : 'Work mode: plan first' },
  ];
}

/** Example messages for an empty chat; clicking one fills the composer. */
export function EmptyChatExamples({ sessionId, memberNames, zh, onPick }: {
  sessionId: string;
  memberNames: string[];
  zh: boolean;
  onPick: (detail: { sessionId: string; text: string; mode: 'free' | 'workflow' }) => void;
}) {
  return (
    <div className="ui-empty-chat">
      <p className="ui-empty-chat-title">{zh ? '试试这样开始' : 'Try one of these'}</p>
      {examplesFor(memberNames, zh).map(example => (
        <button key={example.text} type="button" className="ui-panel ui-empty-chat-example" onClick={() => onPick({ sessionId, text: example.text, mode: example.mode })}>
          <small>{example.note}</small>
          <span>{example.text}</span>
        </button>
      ))}
    </div>
  );
}
