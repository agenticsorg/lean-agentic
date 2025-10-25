#!/usr/bin/env node

/**
 * Run All Theorem Tests
 *
 * Comprehensive test suite runner
 */

const { spawn } = require('child_process');
const path = require('path');

const tests = [
  'basic-theorems.test.js',
  'dependent-types.test.js',
  'performance-theorems.test.js',
];

console.log('🚀 lean-agentic Theorem Test Suite\n');
console.log('='.repeat(60));
console.log(`Running ${tests.length} test suites...\n`);

let currentTest = 0;
let passedSuites = 0;
let failedSuites = 0;

function runNextTest() {
  if (currentTest >= tests.length) {
    console.log('\n' + '='.repeat(60));
    console.log('📊 Final Results:');
    console.log(`   ✅ Passed: ${passedSuites}/${tests.length}`);
    console.log(`   ❌ Failed: ${failedSuites}/${tests.length}`);

    if (failedSuites === 0) {
      console.log('\n🎉 All theorem test suites passed!\n');
      process.exit(0);
    } else {
      console.log(`\n⚠️  ${failedSuites} suite(s) failed\n`);
      process.exit(1);
    }
    return;
  }

  const testFile = tests[currentTest];
  const testPath = path.join(__dirname, testFile);

  console.log(`\n[${ currentTest + 1}/${tests.length}] Running: ${testFile}`);
  console.log('-'.repeat(60));

  const testProcess = spawn('node', [testPath], {
    stdio: 'inherit',
  });

  testProcess.on('close', (code) => {
    if (code === 0) {
      passedSuites++;
    } else {
      failedSuites++;
    }

    currentTest++;
    setTimeout(runNextTest, 500); // Small delay between tests
  });
}

// Start running tests
runNextTest();
