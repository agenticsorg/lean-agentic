# Publishing lean-agentic to NPM

**lean-agentic** - Hash-consed dependent types with 150x faster equality

**Developed by**: [ruv.io](https://ruv.io) | [github.com/ruvnet](https://github.com/ruvnet)

---

## 📦 Package Overview

`lean-agentic` is a WebAssembly-powered npm package that brings formally verified theorem proving to JavaScript/TypeScript with:

- ⚡ **150x faster equality** via hash-consing
- 🛡️ **Dependent types** (Lean4-style type theory)
- 📦 **Zero-copy** arena allocation
- 🌐 **Universal** - Works in Node.js, browsers, Deno, and Bun
- 🎯 **TypeScript** support out of the box
- 🔧 **CLI tools** included (`npx lean-agentic`)

---

## 🏗️ Build Process

### Prerequisites
- Rust 1.90+ with `wasm32-unknown-unknown` target
- Node.js 18+
- wasm-pack (`cargo install wasm-pack`)

### Build Steps

```bash
cd /workspaces/lean-agentic

# 1. Build WASM for all targets
cd leanr-wasm

# Node.js target (CommonJS)
wasm-pack build --target nodejs --out-dir ../npm/lean-agentic/wasm-node

# Web target (ES Modules for browsers)
wasm-pack build --target web --out-dir ../npm/lean-agentic/wasm-web

# Bundler target (for Webpack/Vite/Rollup)
wasm-pack build --target bundler --out-dir ../npm/lean-agentic/wasm

# 2. Navigate to npm package
cd ../npm/lean-agentic

# 3. Install dependencies
npm install

# 4. Test locally
npm run example:node

# 5. Test CLI
node cli/index.js demo
node cli/index.js bench
```

---

## 📋 Pre-Publication Checklist

- [x] WASM binaries built for all targets (node, web, bundler)
- [x] package.json configured with correct metadata
- [x] README.md complete with badges and examples
- [x] TypeScript definitions (.d.ts files) included
- [x] CLI tool tested and working
- [x] Node.js examples working
- [x] Browser examples working
- [x] LICENSE file included
- [x] .npmignore configured properly

---

## 🚀 Publishing Commands

### Test Package Locally

```bash
# Create tarball
npm pack

# Inspect contents
tar -xzf lean-agentic-0.1.0.tgz
ls -la package/

# Test local install
npm install ./lean-agentic-0.1.0.tgz
```

### Dry Run

```bash
npm publish --dry-run
```

### Publish to NPM

```bash
# Login to npm (if not already)
npm login

# Publish (public package)
npm publish --access public

# Or publish specific version
npm publish --tag latest
```

### Verify Publication

```bash
# Check npm registry
npm view lean-agentic

# Install from registry
npm install lean-agentic

# Test global install
npm install -g lean-agentic
lean-agentic --version
```

---

## 📦 Package Structure

```
lean-agentic/
├── package.json          # NPM metadata
├── README.md             # Main documentation with badges
├── LICENSE               # Apache-2.0 license
├── .npmignore            # Files to exclude from publish
├── cli/
│   └── index.js          # CLI entry point (#!/usr/bin/env node)
├── src/
│   ├── index.js          # Main entry (bundler)
│   ├── node.js           # Node.js specific
│   └── web.js            # Browser specific
├── dist/
│   ├── index.d.ts        # TypeScript definitions (bundler)
│   ├── node.d.ts         # TypeScript definitions (Node.js)
│   └── web.d.ts          # TypeScript definitions (browser)
├── wasm/                 # WASM for bundlers
│   ├── leanr_wasm.js
│   ├── leanr_wasm_bg.wasm
│   └── ...
├── wasm-node/            # WASM for Node.js
│   ├── leanr_wasm.js
│   ├── leanr_wasm_bg.wasm
│   └── ...
├── wasm-web/             # WASM for browsers
│   ├── leanr_wasm.js
│   ├── leanr_wasm_bg.wasm
│   └── ...
└── examples/
    ├── node-example.js   # Server-side example
    └── web-example.html  # Browser example
```

---

## 🎯 NPM Scripts

```json
{
  "scripts": {
    "build": "npm run build:wasm && npm run build:js",
    "build:wasm": "cd ../../leanr-wasm && wasm-pack build --target bundler --out-dir ../npm/lean-agentic/wasm",
    "prepublishOnly": "npm run build",
    "test": "node --test",
    "example:node": "node examples/node-example.js",
    "example:web": "npx serve examples"
  }
}
```

---

## 📊 Package Exports

The package supports multiple import styles:

```javascript
// Node.js (CommonJS)
const { createDemo } = require('lean-agentic/node');

// ES Modules (Bundlers)
import { createDemo } from 'lean-agentic';

// Browser (ES Modules)
import { initWeb, createDemo } from 'lean-agentic/web';

// TypeScript
import type { LeanDemo } from 'lean-agentic';
```

---

## 🔧 Maintenance

### Update Version

```bash
# Update version in package.json
npm version patch  # 0.1.0 -> 0.1.1
npm version minor  # 0.1.1 -> 0.2.0
npm version major  # 0.2.0 -> 1.0.0

# Rebuild and publish
npm run build
npm publish
```

### Add New Features

1. Update Rust code in `leanr-wasm/src/lib.rs`
2. Rebuild WASM: `wasm-pack build --target [nodejs|web|bundler]`
3. Update JavaScript wrappers if needed
4. Update TypeScript definitions
5. Test examples
6. Bump version and publish

---

## 📈 Post-Publication

### Verify Package

```bash
# View on NPM
open https://npmjs.com/package/lean-agentic

# Test installation
npm install -g lean-agentic
lean-agentic demo
```

### Monitor Usage

- NPM downloads: https://npmjs.com/package/lean-agentic
- Package size: https://bundlephobia.com/package/lean-agentic
- GitHub stars: https://github.com/agenticsorg/lean-agentic

---

## 🏷️ Tags and Dist-Tags

```bash
# Publish with tag
npm publish --tag beta

# Promote to latest
npm dist-tag add lean-agentic@0.1.0 latest

# List tags
npm dist-tag ls lean-agentic
```

---

## 🔗 Links

- **NPM**: https://npmjs.com/package/lean-agentic
- **Rust Crate**: https://crates.io/crates/lean-agentic
- **Documentation**: https://docs.rs/lean-agentic
- **Repository**: https://github.com/agenticsorg/lean-agentic
- **Homepage**: https://ruv.io

---

## ✅ Final Checklist

Before publishing:

- [ ] All tests pass: `npm test`
- [ ] Examples work: `npm run example:node`
- [ ] CLI works: `node cli/index.js demo`
- [ ] Version bumped in package.json
- [ ] CHANGELOG.md updated
- [ ] README.md badges up to date
- [ ] LICENSE file included
- [ ] Git tagged: `git tag v0.1.0`
- [ ] Pushed to GitHub: `git push --tags`

---

**Built with formal verification** · **Powered by hash-consing** · **Developed by ruv.io**
