/**
 * Simplified AgentDB Integration for lean-agentic
 *
 * Provides in-memory storage with the same API as full AgentDB
 * For production use, see examples/agentdb-full-example.js
 *
 * @author ruv.io
 * @license Apache-2.0
 */

class SimpleLeanAgenticDB {
  constructor(demo, config = {}) {
    this.demo = demo;
    this.config = {
      dbPath: config.dbPath || './lean-theorems.db',
      ...config
    };
    this.storage = {
      theorems: [],
      patterns: new Map(),
      nextId: 1
    };
    this.initialized = false;
  }

  async init() {
    if (this.initialized) return { success: true };

    // Load from file if it exists
    const fs = require('fs');
    if (fs.existsSync(this.config.dbPath)) {
      try {
        const data = JSON.parse(fs.readFileSync(this.config.dbPath, 'utf8'));
        this.storage = data;
        // Restore Map objects
        this.storage.patterns = new Map(Object.entries(data.patterns || {}));
      } catch (error) {
        // If file is corrupt, start fresh
        this.storage = {
          theorems: [],
          patterns: new Map(),
          nextId: 1
        };
      }
    }

    this.initialized = true;
    return { success: true, message: 'Storage initialized' };
  }

  async save() {
    const fs = require('fs');
    const data = {
      ...this.storage,
      patterns: Object.fromEntries(this.storage.patterns)
    };
    fs.writeFileSync(this.config.dbPath, JSON.stringify(data, null, 2), 'utf8');
  }

  async storeTheorem(theorem) {
    await this.init();

    const stored = {
      id: this.storage.nextId++,
      type: theorem.type || 'unknown',
      statement: theorem.statement,
      proof: theorem.proof,
      termId: theorem.termId,
      strategy: theorem.strategy || 'direct',
      success: theorem.success !== false,
      timestamp: new Date().toISOString(),
      ...theorem
    };

    this.storage.theorems.push(stored);

    // Persist to file
    await this.save();

    return {
      id: stored.id,
      stored: true,
      ...stored
    };
  }

  async searchSimilarTheorems(query, options = {}) {
    await this.init();

    const limit = options.limit || 5;
    const threshold = options.threshold || 0.1;

    const results = this.storage.theorems
      .map(t => ({
        ...t,
        similarity: Math.max(
          this.calculateSimilarity(query, t.statement),
          this.calculateSimilarity(query, t.proof),
          this.calculateSimilarity(query, t.type),
          this.calculateSimilarity(query, t.strategy)
        )
      }))
      .filter(t => t.similarity > threshold)
      .sort((a, b) => b.similarity - a.similarity)
      .slice(0, limit);

    return results.map(r => ({
      theorem: r.statement,
      proof: r.proof,
      similarity: r.similarity,
      strategy: r.strategy,
      termId: r.termId
    }));
  }

  calculateSimilarity(query, statement) {
    const q = query.toLowerCase();
    const s = statement.toLowerCase();

    // Check for exact substring match
    if (s.includes(q) || q.includes(s)) {
      return 0.9;
    }

    // Check word overlap
    const qWords = q.split(/\W+/).filter(w => w.length > 0);
    const sWords = s.split(/\W+/).filter(w => w.length > 0);
    const common = qWords.filter(word => sWords.includes(word));

    if (common.length === 0) return 0;
    return common.length / Math.max(qWords.length, sWords.length);
  }

  async learnFromProofs(options = {}) {
    await this.init();

    const successful = this.storage.theorems.filter(t => t.success);

    if (successful.length === 0) {
      return { learned: false, reason: 'No successful proofs to learn from' };
    }

    // Group by strategy
    const patterns = new Map();
    for (const theorem of successful) {
      const key = `${theorem.type}:${theorem.strategy}`;
      if (!patterns.has(key)) {
        patterns.set(key, {
          type: theorem.type,
          strategy: theorem.strategy,
          count: 0,
          examples: []
        });
      }
      const pattern = patterns.get(key);
      pattern.count++;
      pattern.examples.push(theorem.statement);
    }

    const patternArray = Array.from(patterns.values()).map(p => ({
      ...p,
      success_rate: (p.count / successful.length * 100).toFixed(1) + '%',
      confidence: p.count >= 3 ? 'high' : p.count >= 2 ? 'medium' : 'low'
    }));

    this.storage.patterns = patterns;

    // Persist to file
    await this.save();

    return {
      learned: true,
      total_theorems: successful.length,
      patterns_found: patternArray.length,
      patterns: patternArray
    };
  }

  async getProofRecommendations(theorem, options = {}) {
    await this.init();

    const similar = await this.searchSimilarTheorems(theorem, {
      limit: options.similarLimit || 3
    });

    const patterns = Array.from(this.storage.patterns.values());

    const strategies = new Map();
    for (const s of similar) {
      if (!strategies.has(s.strategy)) {
        strategies.set(s.strategy, {
          strategy: s.strategy,
          count: 0,
          avg_similarity: 0,
          examples: []
        });
      }
      const strat = strategies.get(s.strategy);
      strat.count++;
      strat.avg_similarity += s.similarity;
      strat.examples.push(s.theorem);
    }

    const recommended = Array.from(strategies.values()).map(s => ({
      ...s,
      avg_similarity: s.avg_similarity / s.count
    })).sort((a, b) => b.avg_similarity - a.avg_similarity);

    return {
      similar_theorems: similar,
      recommended_strategies: recommended,
      learned_patterns: patterns,
      confidence: similar.length > 0 ? similar[0].similarity : 0
    };
  }

  async getStats() {
    await this.init();

    const total = this.storage.theorems.length;
    const successful = this.storage.theorems.filter(t => t.success).length;

    const byType = {};
    for (const t of this.storage.theorems) {
      byType[t.type] = (byType[t.type] || 0) + 1;
    }

    return {
      total_theorems: total,
      successful_proofs: successful,
      success_rate: total > 0 ? successful / total : 0,
      by_type: Object.entries(byType).map(([type, count]) => ({ type, count })),
      database_size: JSON.stringify(this.storage).length,
      initialized: this.initialized
    };
  }

  async close() {
    // No-op for in-memory storage
    return { success: true };
  }
}

module.exports = { SimpleLeanAgenticDB };
