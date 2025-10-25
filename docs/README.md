# Lean-Agentic Documentation

Complete documentation for the Lean-Agentic formally verified agentic programming language.

## 📚 Documentation Structure

### Quick Start
- [Project README](../README.md) - Overview and quick start guide
- [ARCHITECTURE.md](ARCHITECTURE.md) - High-level system architecture
- [SWARM_IMPLEMENTATION_COMPLETE.md](SWARM_IMPLEMENTATION_COMPLETE.md) - Complete implementation report

### Architecture (Detailed)
- [architecture/README.md](architecture/README.md) - Architecture documentation index
- [architecture/00-overview.md](architecture/00-overview.md) - System design overview
- [architecture/01-memory-model.md](architecture/01-memory-model.md) - Hash-consing and arenas
- [architecture/02-proof-kernel.md](architecture/02-proof-kernel.md) - Trusted computing base
- [architecture/03-performance.md](architecture/03-performance.md) - Performance optimization
- [architecture/04-integration-points.md](architecture/04-integration-points.md) - Component interfaces

### Architecture Decisions (ADRs)
- [decisions/ADR-001-hash-consing.md](decisions/ADR-001-hash-consing.md) - Hash-consing design decision

### Diagrams
- [diagrams/c4-system-context.md](diagrams/c4-system-context.md) - C4 Level 1: System context
- [diagrams/c4-container.md](diagrams/c4-container.md) - C4 Level 2: Container view

### Implementation Guides
- [elaboration-implementation.md](elaboration-implementation.md) - Elaborator technical details
- [WASM_COMPILER_IMPLEMENTATION.md](WASM_COMPILER_IMPLEMENTATION.md) - WASM compiler guide
- [runtime-implementation.md](runtime-implementation.md) - Runtime internals
- [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md) - AI optimization integration

### Summaries
- [ARCHITECTURE-SUMMARY.md](ARCHITECTURE-SUMMARY.md) - Core architecture summary
- [elaboration-summary.md](elaboration-summary.md) - Elaborator summary
- [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) - AI optimization summary
- [RUNTIME_SUMMARY.md](RUNTIME_SUMMARY.md) - Runtime summary

### User Guides
- [PRODUCTION_EXAMPLES.md](PRODUCTION_EXAMPLES.md) - Complete example usage guide
- [RUNBOOK.md](RUNBOOK.md) - Operations and troubleshooting
- [TESTING_SUMMARY.md](TESTING_SUMMARY.md) - Test coverage and benchmarks

## 🎯 Navigation Guide

### For Users
Start here:
1. [Project README](../README.md) - Overview and quick start
2. [PRODUCTION_EXAMPLES.md](PRODUCTION_EXAMPLES.md) - Example usage
3. [RUNBOOK.md](RUNBOOK.md) - Operations guide

### For Contributors
Start here:
1. [ARCHITECTURE.md](ARCHITECTURE.md) - High-level architecture
2. [architecture/](architecture/) - Detailed design docs
3. [SWARM_IMPLEMENTATION_COMPLETE.md](SWARM_IMPLEMENTATION_COMPLETE.md) - Implementation details

### For Researchers
Start here:
1. [architecture/02-proof-kernel.md](architecture/02-proof-kernel.md) - Formal verification
2. [decisions/](decisions/) - Architecture decisions with rationale
3. [architecture/03-performance.md](architecture/03-performance.md) - Performance analysis

## 📊 Documentation Statistics

- **Total Documentation**: 128KB
- **Architecture Docs**: 91KB (9 files)
- **Implementation Guides**: 4 files
- **User Guides**: 3 files (37KB)
- **Diagrams**: 2 C4 models
- **ADRs**: 1 decision record

## 🔧 Building Documentation

```bash
# Generate API documentation
cargo doc --workspace --no-deps --open

# View architecture locally
mdcat docs/ARCHITECTURE.md

# Generate PDF (requires pandoc)
pandoc docs/ARCHITECTURE.md -o architecture.pdf
```

## 📝 Documentation Conventions

- All code examples are tested and working
- Performance numbers are from actual measurements or validated designs
- Architecture diagrams follow C4 model
- ADRs document major design decisions with rationale
- All documentation is kept up-to-date with code

## 🤝 Contributing to Documentation

When adding documentation:
1. Follow the existing structure
2. Add entries to this README
3. Use Markdown with GitHub-flavored syntax
4. Include code examples that compile
5. Measure performance claims
6. Update diagrams if architecture changes

---

**Last Updated**: 2025-10-25
**Documentation Version**: 1.0.0
