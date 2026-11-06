/**
 * AgentDB-backed MCP tools for lean-agentic.
 *
 * agentdb is an optional dependency. It is ESM-only (agentdb >= 3), so it is
 * loaded lazily with a dynamic import the first time an agentdb tool runs.
 * When it is not installed (or the Node version cannot load it), every tool
 * returns a structured "unavailable" result instead of throwing.
 *
 * Embeddings: by default AgentDB's transformers.js provider is used
 * (Xenova/all-MiniLM-L6-v2, 384 dims), which downloads the model on first use.
 * Set LEAN_AGENTIC_EMBEDDINGS=mock (or AGENTDB_DISABLE_TRANSFORMERS=1) to use
 * AgentDB's deterministic offline embeddings instead.
 */

'use strict';

const fs = require('fs');

const DEFAULT_DB_PATH = './lean-theorems.db';
const EMBEDDING_MODEL = 'Xenova/all-MiniLM-L6-v2';
const EMBEDDING_DIMENSION = 384;

const SCHEMA = `
  CREATE TABLE IF NOT EXISTS theorems (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL,
    statement TEXT NOT NULL,
    proof TEXT NOT NULL,
    term_id TEXT,
    strategy TEXT,
    success INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
  );
  CREATE TABLE IF NOT EXISTS theorem_embeddings (
    theorem_id INTEGER PRIMARY KEY,
    embedding BLOB,
    FOREIGN KEY(theorem_id) REFERENCES theorems(id)
  );
`;

const json = (value) => JSON.stringify(value, null, 2);

function defaultLoader() {
  return import('agentdb');
}

function unavailable(error) {
  return json({
    available: false,
    message: 'AgentDB is not available. Install it with `npm install agentdb` (requires Node.js >= 20.9).',
    error: error && error.message ? error.message : String(error),
  });
}

function requireString(args, key) {
  const value = args[key];
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`"${key}" must be a non-empty string`);
  }
  return value;
}

function toBlob(vector) {
  return Buffer.from(vector.buffer, vector.byteOffset, vector.byteLength);
}

function toFloat32(blob) {
  const bytes = blob instanceof Uint8Array ? blob : Buffer.from(blob);
  const copy = new Uint8Array(bytes.byteLength);
  copy.set(bytes);
  return new Float32Array(copy.buffer);
}

class AgentDBTools {
  /**
   * @param {object} [options]
   * @param {() => Promise<object>} [options.loader] - returns the agentdb module
   * @param {'transformers'|'local'} [options.embeddingProvider]
   */
  constructor(options = {}) {
    this.loader = options.loader || defaultLoader;
    this.embeddingProvider = options.embeddingProvider || AgentDBTools.providerFromEnv();
    this._modulePromise = null;
    this._embedder = null;
  }

  static providerFromEnv() {
    const mode = (process.env.LEAN_AGENTIC_EMBEDDINGS || '').toLowerCase();
    return mode === 'mock' || mode === 'local' ? 'local' : 'transformers';
  }

  /** Resolve the agentdb module once; rejections are not cached. */
  async load() {
    if (!this._modulePromise) {
      this._modulePromise = Promise.resolve()
        .then(() => this.loader())
        .then((mod) => {
          const api = mod && mod.createDatabase ? mod : mod && mod.default;
          if (!api || typeof api.createDatabase !== 'function') {
            throw new Error('agentdb module does not export createDatabase');
          }
          return api;
        });
      this._modulePromise.catch(() => {
        this._modulePromise = null;
      });
    }
    return this._modulePromise;
  }

  async isAvailable() {
    try {
      await this.load();
      return true;
    } catch {
      return false;
    }
  }

  async embedder(agentdb) {
    if (!this._embedder) {
      const service = new agentdb.EmbeddingService({
        model: EMBEDDING_MODEL,
        dimension: EMBEDDING_DIMENSION,
        provider: this.embeddingProvider,
      });
      await service.initialize();
      this._embedder = service;
    }
    return this._embedder;
  }

  /** Open the database, run fn, and always close (close persists sql.js DBs). */
  async withDb(agentdb, dbPath, fn) {
    const db = await agentdb.createDatabase(dbPath);
    try {
      return await fn(db);
    } finally {
      db.close();
    }
  }

  async run(fn) {
    let agentdb;
    try {
      agentdb = await this.load();
    } catch (error) {
      return unavailable(error);
    }
    return fn(agentdb);
  }

  init(args = {}) {
    const dbPath = args.path || DEFAULT_DB_PATH;
    return this.run((agentdb) =>
      this.withDb(agentdb, dbPath, (db) => {
        db.exec(SCHEMA);
        return json({
          success: true,
          path: dbPath,
          tables: ['theorems', 'theorem_embeddings'],
          message: 'AgentDB initialized successfully',
        });
      })
    );
  }

  storeTheorem(args = {}) {
    const dbPath = args.path || DEFAULT_DB_PATH;
    const statement = requireString(args, 'statement');
    const proof = requireString(args, 'proof');
    const type = args.type || 'identity';
    return this.run(async (agentdb) => {
      const embedder = await this.embedder(agentdb);
      const embedding = await embedder.embedPassage(statement);
      return this.withDb(agentdb, dbPath, (db) => {
        db.exec(SCHEMA);
        const info = db
          .prepare('INSERT INTO theorems (type, statement, proof, term_id, strategy) VALUES (?, ?, ?, ?, ?)')
          .run(type, statement, proof, 'TermId(2)', 'direct_construction');
        const theoremId = Number(info.lastInsertRowid);
        db.prepare('INSERT OR REPLACE INTO theorem_embeddings (theorem_id, embedding) VALUES (?, ?)')
          .run(theoremId, toBlob(embedding));
        return json({
          success: true,
          theorem_id: theoremId,
          type,
          statement,
          proof,
          embeddings_generated: true,
          embedding_dimension: embedding.length,
        });
      });
    });
  }

  searchTheorems(args = {}) {
    const dbPath = args.path || DEFAULT_DB_PATH;
    const query = requireString(args, 'query');
    const limit = Number.isInteger(args.limit) && args.limit > 0 ? args.limit : 5;
    const threshold = typeof args.threshold === 'number' ? args.threshold : 0.5;
    return this.run(async (agentdb) => {
      const embedder = await this.embedder(agentdb);
      const queryEmbedding = await embedder.embedQuery(query);
      return this.withDb(agentdb, dbPath, (db) => {
        db.exec(SCHEMA);
        const search = new agentdb.WASMVectorSearch(db);
        const rows = db.prepare(
          `SELECT t.id, t.statement, t.proof, t.type, t.strategy, e.embedding
             FROM theorems t JOIN theorem_embeddings e ON e.theorem_id = t.id`
        ).all();
        const theorems = rows
          .map((row) => ({ row, similarity: search.cosineSimilarity(queryEmbedding, toFloat32(row.embedding)) }))
          .filter((hit) => hit.similarity >= threshold)
          .sort((a, b) => b.similarity - a.similarity)
          .slice(0, limit)
          .map(({ row, similarity }) => ({
            similarity: (similarity * 100).toFixed(1) + '%',
            statement: row.statement,
            proof: row.proof,
            type: row.type,
            strategy: row.strategy,
          }));
        return json({ query, results_found: theorems.length, theorems });
      });
    });
  }

  learnPatterns(args = {}) {
    const dbPath = args.path || DEFAULT_DB_PATH;
    return this.run((agentdb) =>
      this.withDb(agentdb, dbPath, (db) => {
        db.exec(SCHEMA);
        const theorems = db.prepare('SELECT * FROM theorems WHERE success = 1').all();
        if (theorems.length === 0) {
          return json({ learned: false, message: 'No theorems to learn from. Store some theorems first.' });
        }
        const strategies = {};
        for (const theorem of theorems) {
          const strategy = theorem.strategy || 'unknown';
          if (!strategies[strategy]) strategies[strategy] = { count: 0, types: new Set() };
          strategies[strategy].count++;
          strategies[strategy].types.add(theorem.type);
        }
        const patterns = Object.entries(strategies).map(([strategy, data]) => ({
          strategy,
          count: data.count,
          theorem_types: Array.from(data.types),
          success_rate: '100.0%',
        }));
        return json({
          learned: true,
          total_theorems: theorems.length,
          patterns_found: patterns.length,
          patterns,
        });
      })
    );
  }

  async counts(agentdb, dbPath) {
    return this.withDb(agentdb, dbPath, (db) => {
      db.exec(SCHEMA);
      return {
        total: db.prepare('SELECT COUNT(*) AS count FROM theorems').get().count,
        byType: db.prepare('SELECT type, COUNT(*) AS count FROM theorems GROUP BY type').all(),
      };
    });
  }

  getStats(args = {}) {
    const dbPath = args.path || DEFAULT_DB_PATH;
    if (!fs.existsSync(dbPath)) {
      return Promise.resolve(json({
        initialized: false,
        message: 'Database not initialized yet. Run agentdb_init first.',
      }));
    }
    return this.run(async (agentdb) => {
      const { total, byType } = await this.counts(agentdb, dbPath);
      return json({
        total_theorems: total,
        database_size: (fs.statSync(dbPath).size / 1024).toFixed(2) + ' KB',
        theorems_by_type: byType,
      });
    });
  }

  statsResource(dbPath = DEFAULT_DB_PATH) {
    if (!fs.existsSync(dbPath)) {
      return Promise.resolve(json({
        initialized: false,
        message: 'Database not initialized. Use agentdb_init tool.',
      }));
    }
    return this.run(async (agentdb) => {
      const { total, byType } = await this.counts(agentdb, dbPath);
      return json({
        total_theorems: total,
        theorems_by_type: byType,
        vector_search_enabled: true,
        learning_enabled: true,
      });
    });
  }
}

module.exports = { AgentDBTools, DEFAULT_DB_PATH, EMBEDDING_DIMENSION };
