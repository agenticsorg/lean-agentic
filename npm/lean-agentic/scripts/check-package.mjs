#!/usr/bin/env node
/**
 * Install lean-agentic into a fresh temp project and check what a user gets:
 * every documented entry point loads under require and import, the CLI runs,
 * and the MCP server's stdout carries nothing but JSON-RPC.
 *
 * Usage:
 *   node scripts/check-package.mjs ./lean-agentic-0.3.4.tgz   # packed tarball
 *   node scripts/check-package.mjs lean-agentic@0.3.4         # from the registry
 */

import { execFileSync, spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const spec = process.argv[2];
if (!spec) {
  console.error('usage: check-package.mjs <tarball | package spec>');
  process.exit(2);
}
const installSpec = existsSync(spec) ? resolve(spec) : spec;

const dir = mkdtempSync(join(tmpdir(), 'lean-agentic-check-'));
let failures = 0;

function pass(name) {
  console.log(`ok   ${name}`);
}

function fail(name, detail) {
  failures++;
  console.log(`FAIL ${name}`);
  if (detail) console.log(String(detail).trim().replace(/^/gm, '     '));
}

function runNode(args, input) {
  return spawnSync(process.execPath, args, {
    cwd: dir,
    input,
    encoding: 'utf8',
    timeout: 60_000
  });
}

try {
  writeFileSync(join(dir, 'package.json'), JSON.stringify({ name: 'check', version: '1.0.0', private: true }));
  execFileSync('npm', ['install', '--no-audit', '--no-fund', '--loglevel=error', installSpec], {
    cwd: dir,
    stdio: ['ignore', 'ignore', 'inherit'],
    timeout: 300_000
  });
  pass(`install ${spec}`);

  // Each case runs in its own process. It must print exactly the marker line
  // on stdout: anything else (e.g. WASM init logs) is a failure.
  const useDemo = `
    const d = m.createDemo();
    for (const k of ['createIdentity', 'createApplication', 'demonstrateHashConsing', 'getStats']) JSON.parse(d[k]());
  `;
  const cases = [
    ['require("lean-agentic")', ['-e', `const m = require('lean-agentic'); ${useDemo} if (typeof m.init !== 'function') throw new Error('init missing'); console.log('MARK')`]],
    ['import "lean-agentic"', ['--input-type=module', '-e', `import * as m from 'lean-agentic'; import def from 'lean-agentic'; ${useDemo} await m.init(); if (def.createDemo !== m.createDemo) throw new Error('default export mismatch'); console.log('MARK')`]],
    ['require("lean-agentic/node")', ['-e', `const m = require('lean-agentic/node'); ${useDemo} console.log('MARK')`]],
    ['import "lean-agentic/node"', ['--input-type=module', '-e', `import * as m from 'lean-agentic/node'; ${useDemo} console.log('MARK')`]],
    ['import "lean-agentic/web"', ['--input-type=module', '-e', `import * as m from 'lean-agentic/web'; for (const k of ['initWeb', 'createDemo', 'setDebug']) if (typeof m[k] !== 'function') throw new Error(k + ' missing'); console.log('MARK')`]],
    ['require("lean-agentic/package.json")', ['-e', `const p = require('lean-agentic/package.json'); if (!p.version) throw new Error('no version'); console.log('MARK')`]],
    ['LEAN_AGENTIC_DEBUG logs to stderr only', ['-e', `process.env.LEAN_AGENTIC_DEBUG = '1'; const m = require('lean-agentic/node'); m.createDemo().createIdentity(); console.log('MARK')`]]
  ];
  for (const [name, args] of cases) {
    const r = runNode(args);
    if (r.status === 0 && r.stdout === 'MARK\n') pass(name);
    else fail(name, `exit ${r.status}\nstdout: ${JSON.stringify(r.stdout)}\nstderr: ${r.stderr}`);
  }

  // CLI
  const bin = join(dir, 'node_modules', 'lean-agentic', 'cli', 'index.js');
  for (const args of [['--version'], ['demo']]) {
    const r = runNode([bin, ...args]);
    if (r.status === 0 && r.stdout.trim()) pass(`cli ${args.join(' ')}`);
    else fail(`cli ${args.join(' ')}`, `exit ${r.status}\n${r.stdout}\n${r.stderr}`);
  }

  // MCP over stdio: initialize, the initialized notification, then tool calls
  // that construct the WASM demo (where the old logs came from).
  const requests = [
    { jsonrpc: '2.0', id: 1, method: 'initialize', params: { protocolVersion: '2024-11-05', capabilities: {}, clientInfo: { name: 'check', version: '1' } } },
    { jsonrpc: '2.0', method: 'notifications/initialized' },
    { jsonrpc: '2.0', id: 2, method: 'tools/call', params: { name: 'create_identity', arguments: {} } },
    { jsonrpc: '2.0', id: 3, method: 'tools/call', params: { name: 'demonstrate_hash_consing', arguments: {} } }
  ];
  const input = requests.map((r) => JSON.stringify(r)).join('\n') + '\n';
  const servers = [
    ['mcp/server.js', [join(dir, 'node_modules', 'lean-agentic', 'mcp', 'server.js')]],
    ['cli mcp start', [bin, 'mcp', 'start']]
  ];
  for (const [name, args] of servers) {
    const out = await runServer(args, input, 3);
    const lines = out.split('\n').filter((l) => l.length > 0);
    const problems = [];
    let first;
    try {
      first = JSON.parse(lines[0]);
    } catch {
      problems.push(`first stdout line is not JSON: ${JSON.stringify(lines[0])}`);
    }
    if (first && !(first.jsonrpc === '2.0' && first.id === 1 && first.result)) {
      problems.push(`first stdout line is not the initialize result: ${lines[0]}`);
    }
    const ids = [];
    for (const line of lines) {
      try {
        const msg = JSON.parse(line);
        if (msg.jsonrpc !== '2.0') problems.push(`not JSON-RPC 2.0: ${line}`);
        if (msg.error) problems.push(`error response: ${line}`);
        ids.push(msg.id);
      } catch {
        problems.push(`non-JSON stdout line: ${JSON.stringify(line)}`);
      }
    }
    if (ids.join(',') !== '1,2,3') problems.push(`expected responses to ids 1,2,3, got ${ids.join(',')}`);
    if (problems.length === 0) pass(`MCP stdio (${name}): ${lines.length} JSON-RPC lines, nothing else`);
    else fail(`MCP stdio (${name})`, problems.join('\n'));
  }
} catch (error) {
  fail('setup', error.stack || error);
} finally {
  rmSync(dir, { recursive: true, force: true });
}

if (failures) {
  console.log(`\n${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nall checks passed');

/**
 * Start the server, write the requests, wait for `expected` responses (or 30s),
 * then close stdin and kill the process. Resolves with everything on stdout.
 */
function runServer(args, input, expected) {
  return new Promise((resolvePromise) => {
    const child = spawn(process.execPath, args, { cwd: dir, stdio: ['pipe', 'pipe', 'ignore'] });
    let out = '';
    const finish = () => {
      clearTimeout(timer);
      child.stdin.end();
      setTimeout(() => {
        child.kill();
        resolvePromise(out);
      }, 200);
    };
    const timer = setTimeout(finish, 30_000);
    child.stdout.on('data', (chunk) => {
      out += chunk;
      if (out.split('\n').filter(Boolean).length >= expected) finish();
    });
    child.on('exit', () => {
      clearTimeout(timer);
      resolvePromise(out);
    });
    child.stdin.write(input);
  });
}
