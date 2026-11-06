'use strict';

// Real-import tests: every code path that touches agentdb runs against the
// installed agentdb package (no mocks of its API). Embeddings use AgentDB's
// deterministic offline provider so the suite never downloads a model.

const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('fs');
const os = require('os');
const path = require('path');

const { AgentDBTools } = require('../../mcp/agentdb-tools.js');
const { LeanAgenticMCPServer } = require('../../mcp/server.js');

const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'lean-agentic-agentdb-'));
test.after(() => fs.rmSync(tmp, { recursive: true, force: true }));

const dbFile = (name) => path.join(tmp, name);
const parse = (text) => JSON.parse(text);

test('agentdb resolves to a 3.x release via a real dynamic import', async () => {
  const pkg = JSON.parse(fs.readFileSync(require.resolve('agentdb/package.json'), 'utf8'));
  assert.match(pkg.version, /^3\./);
  const tools = new AgentDBTools({ embeddingProvider: 'local' });
  const mod = await tools.load();
  for (const name of ['createDatabase', 'EmbeddingService', 'WASMVectorSearch']) {
    assert.equal(typeof mod[name], 'function', `agentdb exports ${name}`);
  }
});

test('init, store, search, learn and stats round-trip through agentdb', async () => {
  const tools = new AgentDBTools({ embeddingProvider: 'local' });
  const db = dbFile('roundtrip.db');

  const init = parse(await tools.init({ path: db }));
  assert.equal(init.success, true);
  assert.ok(fs.existsSync(db), 'database file is persisted on close');

  const empty = parse(await tools.learnPatterns({ path: db }));
  assert.equal(empty.learned, false);

  const first = parse(await tools.storeTheorem({ path: db, statement: 'forall A, A -> A', proof: 'fun x => x' }));
  assert.equal(first.success, true);
  assert.equal(first.theorem_id, 1);
  assert.equal(first.embedding_dimension, 384);
  const second = parse(await tools.storeTheorem({
    path: db, type: 'composition', statement: 'forall A B C, (B -> C) -> (A -> B) -> A -> C', proof: 'fun g f x => g (f x)',
  }));
  assert.equal(second.theorem_id, 2);

  const hit = parse(await tools.searchTheorems({ path: db, query: 'forall A, A -> A', limit: 5 }));
  assert.ok(hit.results_found >= 1);
  assert.equal(hit.theorems[0].statement, 'forall A, A -> A');
  assert.equal(hit.theorems[0].similarity, '100.0%');

  const capped = parse(await tools.searchTheorems({ path: db, query: 'anything', limit: 1, threshold: -1 }));
  assert.equal(capped.results_found, 1);

  const learned = parse(await tools.learnPatterns({ path: db }));
  assert.equal(learned.learned, true);
  assert.equal(learned.total_theorems, 2);
  assert.deepEqual(learned.patterns[0].theorem_types.sort(), ['composition', 'identity']);

  const stats = parse(await tools.getStats({ path: db }));
  assert.equal(stats.total_theorems, 2);
  assert.equal(stats.theorems_by_type.length, 2);

  const resource = parse(await tools.statsResource(db));
  assert.equal(resource.total_theorems, 2);
  assert.equal(resource.vector_search_enabled, true);
});

test('stats report an uninitialized database without touching agentdb', async () => {
  const tools = new AgentDBTools({ loader: () => { throw new Error('must not load'); } });
  const missing = dbFile('missing.db');
  assert.equal(parse(await tools.getStats({ path: missing })).initialized, false);
  assert.equal(parse(await tools.statsResource(missing)).initialized, false);
  assert.ok(!fs.existsSync(missing));
});

test('store and search validate their required arguments', async () => {
  const tools = new AgentDBTools({ embeddingProvider: 'local' });
  assert.throws(() => tools.storeTheorem({ path: dbFile('v.db'), proof: 'p' }), /statement/);
  assert.throws(() => tools.searchTheorems({ path: dbFile('v.db') }), /query/);
});

test('falls back to a structured result when agentdb cannot be loaded', async () => {
  const tools = new AgentDBTools({
    loader: () => Promise.reject(Object.assign(new Error("Cannot find package 'agentdb'"), { code: 'ERR_MODULE_NOT_FOUND' })),
  });
  assert.equal(await tools.isAvailable(), false);
  const db = dbFile('fallback.db');
  for (const result of [
    await tools.init({ path: db }),
    await tools.storeTheorem({ path: db, statement: 's', proof: 'p' }),
    await tools.searchTheorems({ path: db, query: 'q' }),
    await tools.learnPatterns({ path: db }),
  ]) {
    const body = parse(result);
    assert.equal(body.available, false);
    assert.match(body.message, /npm install agentdb/);
  }
  assert.ok(!fs.existsSync(db));
});

test('MCP server routes every agentdb tool and resource through agentdb', async () => {
  const server = new LeanAgenticMCPServer();
  server.agentdb = new AgentDBTools({ embeddingProvider: 'local' });
  const responses = new Map();
  server.sendResponse = (id, result) => responses.set(id, { result });
  server.sendError = (id, code, message) => responses.set(id, { error: { code, message } });
  const db = dbFile('server.db');

  const call = async (id, name, args) => {
    await server.handleToolsCall(id, { name, arguments: args });
    const response = responses.get(id);
    assert.ok(response.result, `${name} failed: ${JSON.stringify(response.error)}`);
    return parse(response.result.content[0].text);
  };

  assert.equal((await call(1, 'agentdb_init', { path: db })).success, true);
  assert.equal((await call(2, 'agentdb_store_theorem', { path: db, statement: 'A -> A', proof: 'id' })).theorem_id, 1);
  assert.equal((await call(3, 'agentdb_search_theorems', { path: db, query: 'A -> A' })).results_found, 1);
  assert.equal((await call(4, 'agentdb_learn_patterns', { path: db })).learned, true);
  assert.equal((await call(5, 'agentdb_get_stats', { path: db })).total_theorems, 1);

  const cwd = process.cwd();
  process.chdir(tmp);
  try {
    fs.copyFileSync(db, path.join(tmp, 'lean-theorems.db'));
    await server.handleResourcesRead(6, { uri: 'stats://agentdb' });
  } finally {
    process.chdir(cwd);
  }
  assert.equal(parse(responses.get(6).result.contents[0].text).total_theorems, 1);
});
