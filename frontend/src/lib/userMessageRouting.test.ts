import assert from 'node:assert/strict';
import { resolveUserMessageMentions } from './userMessageRouting';

assert.deepEqual(resolveUserMessageMentions('记一条笔记', 'free', ['lead']), []);
assert.deepEqual(resolveUserMessageMentions('Uploaded report.pdf', 'free', ['lead']), []);
assert.deepEqual(resolveUserMessageMentions('@开发者 请修改，再让 @reviewer 检查', 'free', ['lead']), ['开发者', 'reviewer']);
assert.deepEqual(resolveUserMessageMentions('@reviewer @reviewer', 'free'), ['reviewer']);
assert.deepEqual(resolveUserMessageMentions('dev@example.com and prefix.@lead', 'free'), []);
assert.deepEqual(resolveUserMessageMentions('执行这个计划', 'workflow', ['lead']), ['lead']);
assert.deepEqual(resolveUserMessageMentions('@reviewer 检查', 'workflow'), ['reviewer']);
assert.deepEqual(resolveUserMessageMentions('执行这个计划', 'workflow'), []);
console.log('Explicit user message routing tests passed');
