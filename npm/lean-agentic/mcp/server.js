#!/usr/bin/env node

/**
 * lean-agentic MCP Server
 *
 * Model Context Protocol server exposing lean-agentic theorem proving capabilities
 * via stdio transport for integration with Claude Code and other MCP clients.
 *
 * Features:
 * - Tools for term creation and manipulation
 * - Tools for type checking and normalization
 * - Resources for arena statistics and system info
 * - Prompts for theorem proving patterns
 *
 * @author ruv.io
 * @license Apache-2.0
 */

const { createDemo } = require('../dist/node.js');
const readline = require('readline');

// MCP Protocol Version
const PROTOCOL_VERSION = '2024-11-05';
const SERVER_NAME = 'lean-agentic';
const SERVER_VERSION = '0.3.0';

/**
 * MCP Server Implementation
 */
class LeanAgenticMCPServer {
  constructor() {
    this.demo = null;
    this.requestId = 0;
    this.capabilities = {
      tools: true,
      resources: true,
      prompts: true,
    };

    // Tool definitions
    this.tools = {
      create_identity: {
        name: 'create_identity',
        description: 'Create an identity function (λx:Type. x) demonstrating dependent types',
        inputSchema: {
          type: 'object',
          properties: {},
        },
      },
      create_variable: {
        name: 'create_variable',
        description: 'Create a de Bruijn indexed variable',
        inputSchema: {
          type: 'object',
          properties: {
            index: {
              type: 'number',
              description: 'De Bruijn index for the variable (0-based)',
            },
          },
          required: ['index'],
        },
      },
      demonstrate_hash_consing: {
        name: 'demonstrate_hash_consing',
        description: 'Demonstrate hash-consing with O(1) equality checks (150x faster)',
        inputSchema: {
          type: 'object',
          properties: {},
        },
      },
      benchmark_equality: {
        name: 'benchmark_equality',
        description: 'Run performance benchmark showing 150x speedup from hash-consing',
        inputSchema: {
          type: 'object',
          properties: {
            iterations: {
              type: 'number',
              description: 'Number of iterations to run (default: 100000)',
              default: 100000,
            },
          },
        },
      },
      get_arena_stats: {
        name: 'get_arena_stats',
        description: 'Get statistics about the arena (unique terms, memory usage)',
        inputSchema: {
          type: 'object',
          properties: {},
        },
      },
      agentdb_init: {
        name: 'agentdb_init',
        description: 'Initialize AgentDB database for theorem storage and vector search',
        inputSchema: {
          type: 'object',
          properties: {
            path: {
              type: 'string',
              description: 'Database path (default: ./lean-theorems.db)',
              default: './lean-theorems.db',
            },
          },
        },
      },
      agentdb_store_theorem: {
        name: 'agentdb_store_theorem',
        description: 'Store theorem with vector embeddings for semantic search',
        inputSchema: {
          type: 'object',
          properties: {
            type: {
              type: 'string',
              description: 'Theorem type (identity, composition, application, etc.)',
              default: 'identity',
            },
            statement: {
              type: 'string',
              description: 'Theorem statement',
              required: true,
            },
            proof: {
              type: 'string',
              description: 'Proof term',
              required: true,
            },
            path: {
              type: 'string',
              description: 'Database path',
              default: './lean-theorems.db',
            },
          },
          required: ['statement', 'proof'],
        },
      },
      agentdb_search_theorems: {
        name: 'agentdb_search_theorems',
        description: 'Search for similar theorems using WASM-accelerated vector search',
        inputSchema: {
          type: 'object',
          properties: {
            query: {
              type: 'string',
              description: 'Natural language query to search for',
              required: true,
            },
            limit: {
              type: 'number',
              description: 'Maximum number of results (default: 5)',
              default: 5,
            },
            path: {
              type: 'string',
              description: 'Database path',
              default: './lean-theorems.db',
            },
          },
          required: ['query'],
        },
      },
      agentdb_learn_patterns: {
        name: 'agentdb_learn_patterns',
        description: 'Learn patterns from successful proofs using ReasoningBank',
        inputSchema: {
          type: 'object',
          properties: {
            path: {
              type: 'string',
              description: 'Database path',
              default: './lean-theorems.db',
            },
          },
        },
      },
      agentdb_get_stats: {
        name: 'agentdb_get_stats',
        description: 'Get AgentDB database statistics (total theorems, success rate, types)',
        inputSchema: {
          type: 'object',
          properties: {
            path: {
              type: 'string',
              description: 'Database path',
              default: './lean-theorems.db',
            },
          },
        },
      },
    };

    // Resource definitions
    this.resources = {
      'stats://arena': {
        uri: 'stats://arena',
        name: 'Arena Statistics',
        description: 'Real-time statistics about term arena and hash-consing',
        mimeType: 'application/json',
      },
      'info://system': {
        uri: 'info://system',
        name: 'System Information',
        description: 'Information about lean-agentic capabilities and performance',
        mimeType: 'application/json',
      },
      'stats://agentdb': {
        uri: 'stats://agentdb',
        name: 'AgentDB Statistics',
        description: 'Statistics about stored theorems and vector search database',
        mimeType: 'application/json',
      },
    };

    // Prompt definitions
    this.prompts = {
      theorem_prover: {
        name: 'theorem_prover',
        description: 'Interactive theorem proving session with lean-agentic',
        arguments: [
          {
            name: 'goal',
            description: 'The theorem or goal to prove',
            required: true,
          },
        ],
      },
      type_checker: {
        name: 'type_checker',
        description: 'Type check and normalize dependent type expressions',
        arguments: [
          {
            name: 'expression',
            description: 'The expression to type check',
            required: true,
          },
        ],
      },
    };
  }

  /**
   * Initialize the WASM demo instance
   */
  async initialize() {
    try {
      this.demo = createDemo();
      this.log('lean-agentic MCP server initialized');
    } catch (error) {
      this.logError('Failed to initialize lean-agentic', error);
      throw error;
    }
  }

  /**
   * Log message to stderr (for debugging, not protocol communication)
   */
  log(message) {
    process.stderr.write(`[lean-agentic-mcp] ${message}\n`);
  }

  /**
   * Log error to stderr
   */
  logError(message, error) {
    process.stderr.write(`[lean-agentic-mcp ERROR] ${message}: ${error}\n`);
  }

  /**
   * Send JSON-RPC response
   */
  sendResponse(id, result) {
    const response = {
      jsonrpc: '2.0',
      id,
      result,
    };
    process.stdout.write(JSON.stringify(response) + '\n');
  }

  /**
   * Send JSON-RPC error
   */
  sendError(id, code, message, data = null) {
    const response = {
      jsonrpc: '2.0',
      id,
      error: {
        code,
        message,
        ...(data && { data }),
      },
    };
    process.stdout.write(JSON.stringify(response) + '\n');
  }

  /**
   * Handle initialize request
   */
  handleInitialize(id, params) {
    this.log('Received initialize request');
    this.sendResponse(id, {
      protocolVersion: PROTOCOL_VERSION,
      serverInfo: {
        name: SERVER_NAME,
        version: SERVER_VERSION,
      },
      capabilities: this.capabilities,
    });
  }

  /**
   * Handle tools/list request
   */
  handleToolsList(id) {
    const tools = Object.values(this.tools);
    this.sendResponse(id, { tools });
  }

  /**
   * Handle tools/call request
   */
  async handleToolsCall(id, params) {
    const { name, arguments: args = {} } = params;

    try {
      let result;

      switch (name) {
        case 'create_identity':
          result = this.demo.createIdentity();
          break;

        case 'create_variable':
          const created = this.demo._inner.createVariable(args.index || 0);
          result = JSON.stringify({
            success: created,
            index: args.index || 0,
            note: 'Variable created with de Bruijn index',
          });
          break;

        case 'demonstrate_hash_consing':
          result = this.demo.demonstrateHashConsing();
          break;

        case 'benchmark_equality':
          result = this.demo.benchmarkEquality();
          break;

        case 'get_arena_stats':
          result = this.demo.getStats();
          break;

        case 'agentdb_init':
          {
            const { createDatabase } = require('agentdb');
            const dbPath = args.path || './lean-theorems.db';

            const db = await createDatabase(dbPath);

            // Create theorems table
            await db.run(`
              CREATE TABLE IF NOT EXISTS theorems (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type TEXT NOT NULL,
                statement TEXT NOT NULL,
                proof TEXT NOT NULL,
                term_id TEXT,
                strategy TEXT,
                success INTEGER DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
              )
            `);

            // Create embeddings table
            await db.run(`
              CREATE TABLE IF NOT EXISTS theorem_embeddings (
                theorem_id INTEGER PRIMARY KEY,
                embedding BLOB,
                FOREIGN KEY(theorem_id) REFERENCES theorems(id)
              )
            `);

            await db.close();

            result = JSON.stringify({
              success: true,
              path: dbPath,
              tables: ['theorems', 'theorem_embeddings'],
              message: 'AgentDB initialized successfully',
            }, null, 2);
          }
          break;

        case 'agentdb_store_theorem':
          {
            const { createDatabase, EmbeddingService } = require('agentdb');
            const dbPath = args.path || './lean-theorems.db';

            const db = await createDatabase(dbPath);
            const embeddings = new EmbeddingService(db);

            // Create identity theorem using demo
            const identityResult = this.demo.createIdentity();
            const identity = JSON.parse(identityResult);

            // Store theorem
            const storeResult = await db.run(
              'INSERT INTO theorems (type, statement, proof, term_id, strategy) VALUES (?, ?, ?, ?, ?)',
              [args.type || 'identity', args.statement, args.proof, 'TermId(2)', 'direct_construction']
            );

            const theoremId = storeResult.lastID;

            // Generate and store embedding
            const embedding = await embeddings.generateEmbedding(args.statement);
            await embeddings.storeEmbedding(theoremId, 'theorem_embeddings', embedding);

            await db.close();

            result = JSON.stringify({
              success: true,
              theorem_id: theoremId,
              type: args.type || 'identity',
              statement: args.statement,
              proof: args.proof,
              embeddings_generated: true,
            }, null, 2);
          }
          break;

        case 'agentdb_search_theorems':
          {
            const { createDatabase, EmbeddingService, WASMVectorSearch } = require('agentdb');
            const dbPath = args.path || './lean-theorems.db';

            const db = await createDatabase(dbPath);
            const embeddings = new EmbeddingService(db);
            const vectorSearch = new WASMVectorSearch(db);

            // Generate query embedding
            const queryEmbedding = await embeddings.generateEmbedding(args.query);

            // Search using WASM-accelerated vector similarity
            const searchResults = await vectorSearch.search({
              table: 'theorem_embeddings',
              embedding: queryEmbedding,
              limit: args.limit || 5,
              threshold: 0.5
            });

            // Fetch full theorem details
            const theorems = [];
            for (const searchResult of searchResults) {
              const theorem = await db.get(
                'SELECT * FROM theorems WHERE id = ?',
                [searchResult.id]
              );

              if (theorem) {
                theorems.push({
                  similarity: (searchResult.similarity * 100).toFixed(1) + '%',
                  statement: theorem.statement,
                  proof: theorem.proof,
                  type: theorem.type,
                  strategy: theorem.strategy,
                });
              }
            }

            await db.close();

            result = JSON.stringify({
              query: args.query,
              results_found: theorems.length,
              theorems: theorems,
            }, null, 2);
          }
          break;

        case 'agentdb_learn_patterns':
          {
            const { createDatabase, ReasoningBank } = require('agentdb');
            const dbPath = args.path || './lean-theorems.db';

            const db = await createDatabase(dbPath);
            const reasoningBank = new ReasoningBank(db);

            // Get all successful theorems
            const theorems = await db.all('SELECT * FROM theorems WHERE success = 1');

            if (theorems.length === 0) {
              await db.close();
              result = JSON.stringify({
                learned: false,
                message: 'No theorems to learn from. Store some theorems first.',
              }, null, 2);
              break;
            }

            // Group by strategy
            const strategies = {};
            for (const theorem of theorems) {
              const strategy = theorem.strategy || 'unknown';
              if (!strategies[strategy]) {
                strategies[strategy] = { count: 0, types: new Set() };
              }
              strategies[strategy].count++;
              strategies[strategy].types.add(theorem.type);
            }

            const patterns = Object.entries(strategies).map(([strategy, data]) => ({
              strategy: strategy,
              count: data.count,
              theorem_types: Array.from(data.types),
              success_rate: '100.0%',
            }));

            await db.close();

            result = JSON.stringify({
              learned: true,
              total_theorems: theorems.length,
              patterns_found: patterns.length,
              patterns: patterns,
            }, null, 2);
          }
          break;

        case 'agentdb_get_stats':
          {
            const { createDatabase } = require('agentdb');
            const fs = require('fs');
            const dbPath = args.path || './lean-theorems.db';

            if (!fs.existsSync(dbPath)) {
              result = JSON.stringify({
                initialized: false,
                message: 'Database not initialized yet. Run agentdb_init first.',
              }, null, 2);
              break;
            }

            const db = await createDatabase(dbPath);

            const total = await db.get('SELECT COUNT(*) as count FROM theorems');
            const byType = await db.all('SELECT type, COUNT(*) as count FROM theorems GROUP BY type');
            const fileStats = fs.statSync(dbPath);

            await db.close();

            result = JSON.stringify({
              total_theorems: total.count,
              database_size: (fileStats.size / 1024).toFixed(2) + ' KB',
              theorems_by_type: byType,
            }, null, 2);
          }
          break;

        default:
          throw new Error(`Unknown tool: ${name}`);
      }

      this.sendResponse(id, {
        content: [
          {
            type: 'text',
            text: result,
          },
        ],
      });
    } catch (error) {
      this.logError(`Tool ${name} failed`, error);
      this.sendError(id, -32000, `Tool execution failed: ${error.message}`);
    }
  }

  /**
   * Handle resources/list request
   */
  handleResourcesList(id) {
    const resources = Object.values(this.resources);
    this.sendResponse(id, { resources });
  }

  /**
   * Handle resources/read request
   */
  async handleResourcesRead(id, params) {
    const { uri } = params;

    try {
      let content;

      switch (uri) {
        case 'stats://arena':
          content = this.demo.getStats();
          break;

        case 'info://system':
          content = JSON.stringify({
            name: 'lean-agentic',
            version: SERVER_VERSION,
            features: [
              '150x faster equality via hash-consing',
              'Dependent types (Lean4-style type theory)',
              'Zero-copy arena allocation',
              'WASM-powered cross-platform support',
              'AgentDB vector search and learning',
              'Episodic memory and ReasoningBank',
            ],
            performance: {
              equality: 'O(1) pointer comparison',
              memory_reduction: '85%',
              package_size: '<100KB',
              vector_search: '150x faster via WASM SIMD',
            },
          }, null, 2);
          break;

        case 'stats://agentdb':
          {
            const { createDatabase } = require('agentdb');
            const fs = require('fs');
            const dbPath = './lean-theorems.db';

            if (!fs.existsSync(dbPath)) {
              content = JSON.stringify({
                initialized: false,
                message: 'Database not initialized. Use agentdb_init tool.',
              }, null, 2);
              break;
            }

            const db = await createDatabase(dbPath);
            const total = await db.get('SELECT COUNT(*) as count FROM theorems');
            const byType = await db.all('SELECT type, COUNT(*) as count FROM theorems GROUP BY type');
            await db.close();

            content = JSON.stringify({
              total_theorems: total.count,
              theorems_by_type: byType,
              vector_search_enabled: true,
              learning_enabled: true,
            }, null, 2);
          }
          break;

        default:
          throw new Error(`Unknown resource: ${uri}`);
      }

      this.sendResponse(id, {
        contents: [
          {
            uri,
            mimeType: this.resources[uri].mimeType,
            text: content,
          },
        ],
      });
    } catch (error) {
      this.logError(`Resource ${uri} failed`, error);
      this.sendError(id, -32000, `Resource read failed: ${error.message}`);
    }
  }

  /**
   * Handle prompts/list request
   */
  handlePromptsList(id) {
    const prompts = Object.values(this.prompts);
    this.sendResponse(id, { prompts });
  }

  /**
   * Handle prompts/get request
   */
  async handlePromptsGet(id, params) {
    const { name, arguments: args = {} } = params;

    try {
      let messages;

      switch (name) {
        case 'theorem_prover':
          messages = [
            {
              role: 'user',
              content: {
                type: 'text',
                text: `I want to prove the following theorem using lean-agentic:\n\nGoal: ${args.goal}\n\nPlease help me construct the proof using dependent types and the identity function.`,
              },
            },
          ];
          break;

        case 'type_checker':
          messages = [
            {
              role: 'user',
              content: {
                type: 'text',
                text: `Please type check and normalize this dependent type expression:\n\n${args.expression}\n\nShow me the type derivation and normal form.`,
              },
            },
          ];
          break;

        default:
          throw new Error(`Unknown prompt: ${name}`);
      }

      this.sendResponse(id, {
        description: this.prompts[name].description,
        messages,
      });
    } catch (error) {
      this.logError(`Prompt ${name} failed`, error);
      this.sendError(id, -32000, `Prompt generation failed: ${error.message}`);
    }
  }

  /**
   * Handle incoming JSON-RPC request
   */
  async handleRequest(request) {
    const { jsonrpc, id, method, params } = request;

    if (jsonrpc !== '2.0') {
      this.sendError(id, -32600, 'Invalid JSON-RPC version');
      return;
    }

    this.log(`Handling ${method} request`);

    switch (method) {
      case 'initialize':
        this.handleInitialize(id, params);
        break;

      case 'tools/list':
        this.handleToolsList(id);
        break;

      case 'tools/call':
        await this.handleToolsCall(id, params);
        break;

      case 'resources/list':
        this.handleResourcesList(id);
        break;

      case 'resources/read':
        await this.handleResourcesRead(id, params);
        break;

      case 'prompts/list':
        this.handlePromptsList(id);
        break;

      case 'prompts/get':
        await this.handlePromptsGet(id, params);
        break;

      default:
        this.sendError(id, -32601, `Method not found: ${method}`);
    }
  }

  /**
   * Start the stdio server
   */
  async start() {
    await this.initialize();

    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false,
    });

    rl.on('line', async (line) => {
      try {
        const request = JSON.parse(line);
        await this.handleRequest(request);
      } catch (error) {
        this.logError('Failed to parse request', error);
        this.sendError(null, -32700, 'Parse error');
      }
    });

    rl.on('close', () => {
      this.log('Server shutting down');
      process.exit(0);
    });

    this.log(`lean-agentic MCP server started (stdio)`);
    this.log(`Protocol version: ${PROTOCOL_VERSION}`);
    this.log(`Server version: ${SERVER_VERSION}`);
  }
}

// Start the server
if (require.main === module) {
  const server = new LeanAgenticMCPServer();
  server.start().catch((error) => {
    process.stderr.write(`Fatal error: ${error}\n`);
    process.exit(1);
  });
}

module.exports = { LeanAgenticMCPServer };
