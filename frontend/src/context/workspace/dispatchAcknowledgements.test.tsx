import assert from 'node:assert/strict';
import React, { act } from 'react';
import { createRoot } from 'react-dom/client';
import { JSDOM } from 'jsdom';
import { useDispatchAcknowledgements } from './useWorkspaceChatRuntime';

const dom = new JSDOM('<div id="root"></div>');
Object.assign(globalThis, { window: dom.window, document: dom.window.document, IS_REACT_ACT_ENVIRONMENT: true });
let state!: ReturnType<typeof useDispatchAcknowledgements>;
function Probe() { state = useDispatchAcknowledgements(); return null; }
const root = createRoot(document.getElementById('root')!);
await act(async () => root.render(<Probe />));
let firstCalls = 0;
let secondCalls = 0;
await act(async () => {
  state.requestDispatchAcknowledgement('session', { active_count: 1, items: [] }, async () => { firstCalls++; });
  state.requestDispatchAcknowledgement('session', { active_count: 2, items: [] }, async () => { secondCalls++; throw new Error('offline'); });
});
assert.equal(firstCalls, 0);
assert.equal(secondCalls, 0, 'blocked sends never silently retry');
assert.equal(state.dispatchAcknowledgements.session.pending.active_count, 1);
await act(async () => {
  const confirm = state.dispatchAcknowledgements.session.retry;
  await Promise.all([confirm(), confirm()]);
});
assert.equal(firstCalls, 1, 'double clicks submit once');
assert.equal(secondCalls, 0, 'confirming one never dispatches the next');
assert.equal(state.dispatchAcknowledgements.session.pending.active_count, 2);
await act(async () => state.dispatchAcknowledgements.session.retry());
assert.equal(state.dispatchAcknowledgements.session.error, 'offline');
assert.equal(state.dispatchAcknowledgements.session.busy, false);
await act(async () => root.unmount());
dom.window.close();
console.log('dispatch acknowledgement queue tests passed');
