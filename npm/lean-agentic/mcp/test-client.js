#!/usr/bin/env node

/**
 * lean-agentic MCP Test Client
 *
 * Simple test client for manually testing the MCP server
 * Sends JSON-RPC requests and displays responses
 */

const { spawn } = require('child_process');
const readline = require('readline');

class MCPTestClient {
  constructor() {
    this.requestId = 1;
    this.server = null;
  }

  /**
   * Start the MCP server
   */
  async start() {
    console.log('🚀 Starting lean-agentic MCP server...\n');

    this.server = spawn('node', ['./mcp/server.js'], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    this.server.stdout.on('data', (data) => {
      const lines = data.toString().split('\n').filter(l => l.trim());
      lines.forEach(line => {
        try {
          const response = JSON.parse(line);
          console.log('📩 Response:', JSON.stringify(response, null, 2), '\n');
        } catch (e) {
          console.log('Raw output:', line);
        }
      });
    });

    this.server.stderr.on('data', (data) => {
      console.log('🔍 Server log:', data.toString());
    });

    this.server.on('close', (code) => {
      console.log(`Server exited with code ${code}`);
      process.exit(code);
    });

    // Wait a bit for server to start
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  /**
   * Send a JSON-RPC request
   */
  sendRequest(method, params = {}) {
    const request = {
      jsonrpc: '2.0',
      id: this.requestId++,
      method,
      params,
    };

    console.log('📤 Request:', JSON.stringify(request, null, 2), '\n');
    this.server.stdin.write(JSON.stringify(request) + '\n');
  }

  /**
   * Run test sequence
   */
  async runTests() {
    console.log('='.repeat(60));
    console.log('lean-agentic MCP Server Test Suite');
    console.log('='.repeat(60), '\n');

    // Test 1: Initialize
    console.log('Test 1: Initialize server');
    this.sendRequest('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: {
        name: 'test-client',
        version: '1.0.0',
      },
    });

    await this.wait(500);

    // Test 2: List tools
    console.log('Test 2: List available tools');
    this.sendRequest('tools/list');

    await this.wait(500);

    // Test 3: Call create_identity tool
    console.log('Test 3: Call create_identity tool');
    this.sendRequest('tools/call', {
      name: 'create_identity',
      arguments: {},
    });

    await this.wait(500);

    // Test 4: Call demonstrate_hash_consing tool
    console.log('Test 4: Call demonstrate_hash_consing tool');
    this.sendRequest('tools/call', {
      name: 'demonstrate_hash_consing',
      arguments: {},
    });

    await this.wait(500);

    // Test 5: Call get_arena_stats tool
    console.log('Test 5: Call get_arena_stats tool');
    this.sendRequest('tools/call', {
      name: 'get_arena_stats',
      arguments: {},
    });

    await this.wait(500);

    // Test 6: List resources
    console.log('Test 6: List available resources');
    this.sendRequest('resources/list');

    await this.wait(500);

    // Test 7: Read arena stats resource
    console.log('Test 7: Read arena stats resource');
    this.sendRequest('resources/read', {
      uri: 'stats://arena',
    });

    await this.wait(500);

    // Test 8: Read system info resource
    console.log('Test 8: Read system info resource');
    this.sendRequest('resources/read', {
      uri: 'info://system',
    });

    await this.wait(500);

    // Test 9: List prompts
    console.log('Test 9: List available prompts');
    this.sendRequest('prompts/list');

    await this.wait(500);

    // Test 10: Get theorem prover prompt
    console.log('Test 10: Get theorem prover prompt');
    this.sendRequest('prompts/get', {
      name: 'theorem_prover',
      arguments: {
        goal: 'forall A, A -> A (identity theorem)',
      },
    });

    await this.wait(500);

    console.log('\n' + '='.repeat(60));
    console.log('✅ All tests completed!');
    console.log('='.repeat(60), '\n');

    // Shutdown
    this.server.kill();
  }

  /**
   * Wait for specified milliseconds
   */
  wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Run tests
if (require.main === module) {
  const client = new MCPTestClient();
  client.start().then(() => client.runTests()).catch(console.error);
}

module.exports = { MCPTestClient };
