#!/usr/bin/env node

/**
 * Build script for lean-agentic npm package
 * Creates both CommonJS and ESM bundles
 */

const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, '..', 'src');
const distDir = path.join(__dirname, '..', 'dist');

// Ensure dist directory exists
if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

// Files to build
const files = [
  { name: 'index', hasCommonJS: true },
  { name: 'node', hasCommonJS: true },
  { name: 'web', hasCommonJS: false } // Web only needs ESM
];

console.log('📦 Building lean-agentic...\n');

files.forEach(({ name, hasCommonJS }) => {
  const srcFile = path.join(srcDir, `${name}.js`);

  if (!fs.existsSync(srcFile)) {
    console.warn(`⚠️  Warning: ${srcFile} not found, skipping...`);
    return;
  }

  const content = fs.readFileSync(srcFile, 'utf-8');

  // Create ESM version (.mjs)
  const mjsFile = path.join(distDir, `${name}.mjs`);
  fs.writeFileSync(mjsFile, content);
  console.log(`✅ Created ${name}.mjs`);

  // Create CommonJS version (.js) if needed
  if (hasCommonJS) {
    const cjsFile = path.join(distDir, `${name}.js`);
    const cjsContent = convertToCommonJS(content, name);
    fs.writeFileSync(cjsFile, cjsContent);
    console.log(`✅ Created ${name}.js`);
  }
});

console.log('\n✨ Build complete!\n');
console.log('📁 Output:');
console.log('  - dist/*.mjs (ES Modules)');
console.log('  - dist/*.js  (CommonJS)');
console.log('  - dist/*.d.ts (TypeScript definitions)\n');

/**
 * Convert ES Module syntax to CommonJS
 */
function convertToCommonJS(content, moduleName) {
  // Handle different module types
  if (moduleName === 'node') {
    // node.js already uses CommonJS, just copy it
    return content;
  }

  if (moduleName === 'web') {
    // Web module doesn't need CommonJS
    return content;
  }

  // For index.js (bundler), convert import/export to require/module.exports
  let cjs = content
    // Convert: import ... from '../wasm/...'
    .replace(/import\s+(\w+|\{[^}]+\}|\*\s+as\s+\w+)\s+from\s+['"]([^'"]+)['"]/g,
      (match, imports, path) => {
        if (imports.startsWith('*')) {
          const varName = imports.match(/\*\s+as\s+(\w+)/)[1];
          return `const ${varName} = require('${path}')`;
        } else if (imports.startsWith('{')) {
          return `const ${imports} = require('${path}')`;
        } else {
          return `const ${imports} = require('${path}')`;
        }
      })
    // Convert: export class/function
    .replace(/export\s+(class|function|const|let|var)\s+(\w+)/g, '$1 $2')
    // Convert: export default
    .replace(/export\s+default\s+/g, 'module.exports = ')
    // Convert: export { ... }
    .replace(/export\s*\{([^}]+)\}/g, (match, exports) => {
      const items = exports.split(',').map(e => e.trim());
      return `module.exports = { ${items.join(', ')} }`;
    });

  return cjs;
}
