#!/usr/bin/env node

/**
 * lean-agentic + AgentDB Integration Example
 *
 * Demonstrates:
 * - Storing theorems in vector database
 * - Searching for similar theorems
 * - Learning from successful proofs
 * - Getting AI-powered proof recommendations
 *
 * @author ruv.io
 * @license Apache-2.0
 */

const { createDemo } = require('../dist/node.js');
const { LeanAgenticDB } = require('../src/agentdb-integration.js');

async function main() {
  console.log('\n🚀 lean-agentic + AgentDB Integration Demo\n');
  console.log('Combining theorem proving with AI-powered learning!\n');
  console.log('='.repeat(60));

  // Step 1: Initialize
  console.log('\n📚 Step 1: Initialize lean-agentic and AgentDB\n');

  const demo = createDemo();
  const db = new LeanAgenticDB(demo, {
    dbPath: './examples/lean-agentic-demo.db',
    episodicMemory: true,
    reasoningBank: true
  });

  await db.init();
  console.log('✅ Initialized successfully!');

  // Step 2: Store some theorems
  console.log('\n📝 Step 2: Store theorems in vector database\n');

  const theorems = [
    {
      type: 'identity',
      statement: '∀A. A → A',
      proof: 'λx:A. x',
      termId: 'TermId(2)',
      strategy: 'direct_construction',
      success: true
    },
    {
      type: 'composition',
      statement: '∀A B C. (A → B) → (B → C) → (A → C)',
      proof: 'λf g x. g (f x)',
      termId: 'TermId(5)',
      strategy: 'composition',
      success: true
    },
    {
      type: 'application',
      statement: '(A → B) → A → B',
      proof: 'λf x. f x',
      termId: 'TermId(3)',
      strategy: 'application',
      success: true
    }
  ];

  for (const theorem of theorems) {
    const stored = await db.storeTheorem(theorem);
    console.log(`✅ Stored: ${theorem.statement}`);
    console.log(`   ID: ${stored.id} | Strategy: ${stored.strategy}`);
  }

  // Step 3: Search for similar theorems
  console.log('\n🔍 Step 3: Search for similar theorems\n');

  const searchQuery = 'function that takes input and returns it';
  console.log(`Query: "${searchQuery}"\n`);

  const similar = await db.searchSimilarTheorems(searchQuery, { limit: 2 });

  similar.forEach((result, i) => {
    console.log(`${i + 1}. ${result.theorem}`);
    console.log(`   Similarity: ${(result.similarity * 100).toFixed(1)}%`);
    console.log(`   Proof: ${result.proof}`);
    console.log(`   Strategy: ${result.strategy}\n`);
  });

  // Step 4: Record proof attempts (episodic memory)
  console.log('📊 Step 4: Record proof attempts (Episodic Memory)\n');

  const proofAttempts = [
    {
      theorem: '∀A. A → A',
      strategy: 'direct_construction',
      steps: ['introduce variable', 'return variable'],
      success: true,
      duration: 0.5
    },
    {
      theorem: '∀A B. A → B → A',
      strategy: 'direct_construction',
      steps: ['introduce x', 'introduce y', 'return x'],
      success: true,
      duration: 0.8
    }
  ];

  for (const attempt of proofAttempts) {
    const recorded = await db.recordProofAttempt(attempt);
    console.log(`✅ Recorded: ${attempt.theorem}`);
    console.log(`   Episode ID: ${recorded.episode_id}`);
    console.log(`   Success: ${attempt.success} | Duration: ${attempt.duration}s\n`);
  }

  // Step 5: Learn from successful proofs (ReasoningBank)
  console.log('🧠 Step 5: Learn from proofs (ReasoningBank)\n');

  const learning = await db.learnFromProofs({ limit: 10 });

  if (learning.learned) {
    console.log(`✅ Learning complete!`);
    console.log(`   Patterns discovered: ${learning.patterns_found}\n`);

    if (learning.patterns && learning.patterns.length > 0) {
      console.log('📊 Discovered Patterns:\n');
      learning.patterns.slice(0, 3).forEach((pattern, i) => {
        console.log(`${i + 1}. ${pattern.type} using ${pattern.strategy}`);
        console.log(`   Occurrences: ${pattern.count}`);
        console.log(`   Success Rate: ${(pattern.success_rate * 100).toFixed(1)}%`);
        console.log(`   Confidence: ${pattern.confidence}\n`);
      });
    }
  } else {
    console.log(`ℹ️  ${learning.reason}\n`);
  }

  // Step 6: Get proof recommendations
  console.log('💡 Step 6: Get AI-powered proof recommendations\n');

  const newTheorem = 'Prove that identity composition is identity';

  const recommendations = await db.getProofRecommendations(newTheorem, {
    similarLimit: 3,
    patternLimit: 3
  });

  console.log(`For theorem: "${newTheorem}"\n`);

  if (recommendations.similar_theorems.length > 0) {
    console.log('Similar theorems:');
    recommendations.similar_theorems.forEach((t, i) => {
      console.log(`  ${i + 1}. ${t.theorem} (${(t.similarity * 100).toFixed(1)}% similar)`);
    });
    console.log();
  }

  if (recommendations.recommended_strategies.length > 0) {
    console.log('Recommended strategies:');
    recommendations.recommended_strategies.forEach((s, i) => {
      console.log(`  ${i + 1}. ${s.strategy}`);
      console.log(`     Avg Similarity: ${(s.avg_similarity * 100).toFixed(1)}%`);
      console.log(`     Used ${s.count} time(s)`);
    });
    console.log();
  }

  console.log(`Recommendation confidence: ${(recommendations.confidence * 100).toFixed(1)}%\n`);

  // Step 7: Show statistics
  console.log('📈 Step 7: Database statistics\n');

  const stats = await db.getStats();

  console.log(`Total theorems: ${stats.total_theorems}`);
  console.log(`Successful proofs: ${stats.successful_proofs}`);
  console.log(`Success rate: ${(stats.success_rate * 100).toFixed(1)}%`);

  if (stats.by_type && stats.by_type.length > 0) {
    console.log('\nTheorems by type:');
    stats.by_type.forEach(type => {
      console.log(`  • ${type.theorem_type}: ${type.count}`);
    });
  }

  console.log(`\nDatabase size: ${(stats.database_size / 1024).toFixed(2)} KB`);

  // Cleanup
  await db.close();

  console.log('\n' + '='.repeat(60));
  console.log('\n✅ Demo complete! AgentDB integration working perfectly.\n');
  console.log('Features demonstrated:');
  console.log('  ✓ Theorem storage with vector embeddings');
  console.log('  ✓ Semantic similarity search');
  console.log('  ✓ Episodic memory for proof attempts');
  console.log('  ✓ ReasoningBank pattern learning');
  console.log('  ✓ AI-powered proof recommendations\n');
}

// Run the demo
main().catch(error => {
  console.error('\n❌ Error:', error.message);
  console.error(error.stack);
  process.exit(1);
});
