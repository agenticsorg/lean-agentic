# lean-agentic Theorem Implementation Roadmap

**Version**: 1.0
**Date**: 2025-10-25
**Status**: Planning Phase

---

## 🎯 Mission Statement

Implement **5 state-of-the-art theorems** in type theory and formal verification that showcase lean-agentic's unique capabilities:
- 150x hash-consing speedup
- Sub-100ms compilation
- WASM browser deployment
- AI-assisted proof search

---

## 📊 Visual Roadmap

```
                        LEAN-AGENTIC THEOREM ROADMAP
                        ============================

WEEKS 1-2      WEEKS 3-5        WEEKS 6-9           WEEKS 10-16 (Optional)
━━━━━━━━━━     ━━━━━━━━━━       ━━━━━━━━━━          ━━━━━━━━━━━━━━━━━━━━━━

┌─────────┐    ┌─────────┐      ┌──────────┐        ┌──────────┐
│ Church- │───▶│   NbE   │─────▶│Parametri-│───────▶│ π₁(S¹)≃ℤ │
│ Rosser  │    │   for   │      │  city    │        │  (HoTT)  │
│ (Conflu-│    │Dependent│      │ Theorem  │        │          │
│  ence)  │    │  Types  │      │          │        │ Strong   │
└─────────┘    └─────────┘      └──────────┘        │Normaliz- │
                                                     │  ation   │
Difficulty: ⭐⭐⭐⭐⭐⭐        ⭐⭐⭐⭐⭐⭐⭐              ⭐⭐⭐⭐⭐⭐⭐⭐          └──────────┘
                                                     ⭐⭐⭐⭐⭐⭐⭐⭐⭐
LOC:       1,000         1,200           1,300           2,000+

Impact:    Foundation    Practical       Cutting-edge    Research
           + Quick Win   + Performance   + Unique        Prestige
```

---

## 📈 Implementation Timeline

### Phase 1: Foundation (Weeks 1-2) ✅ RECOMMENDED START

**Theorem**: Church-Rosser Confluence
**Goal**: Prove βδιζ-reduction is confluent
**Status**: Ready to implement

```
Week 1: Parallel Reduction
├── Day 1-2: Define parallel reduction relation
├── Day 3-4: Implement compatibility rules
├── Day 5:   Unit tests (20 cases)
└── Day 6-7: Code review & documentation

Week 2: Diamond Property
├── Day 1-2: Implement common reduct finder
├── Day 3-4: Prove diamond lemma
├── Day 5:   Performance benchmarks
├── Day 6:   WASM browser demo
└── Day 7:   Final testing & release
```

**Deliverables**:
- ✅ Confluence checker module
- ✅ 20+ unit tests
- ✅ Performance benchmarks
- ✅ Browser demo (WASM)
- ✅ Documentation

**Success Metrics**:
- <5ms confluence checks
- 150x speedup vs naive
- 100% test pass rate

---

### Phase 2: Core (Weeks 3-5)

**Theorem**: Normalization by Evaluation
**Goal**: Efficient βη-normalization via semantic evaluation
**Status**: Depends on Phase 1

```
Week 3: Semantic Domain
├── Define Value types (VLam, VPi, VNeutral, VSort)
├── Implement Closure type
├── Build evaluation environment
└── Unit tests for domain

Week 4: Evaluation
├── Implement eval: Term → Value
├── Handle lambda, pi, app, var
├── Add evaluation cache
└── Benchmark evaluation

Week 5: Reification
├── Implement reify: Value → Term
├── Handle fresh variable generation
├── Integrate with type checker
└── End-to-end tests
```

**Deliverables**:
- ✅ NbE module (`lean-agentic/src/nbe/`)
- ✅ 30+ unit tests
- ✅ Integration with type checker
- ✅ Performance comparison

**Success Metrics**:
- <1ms normalization
- >95% cache hit rate
- 150x faster than naive

---

### Phase 3: Advanced (Weeks 6-9)

**Theorem**: Parametricity for Dependent Types
**Goal**: Automatic free theorem generation
**Status**: Depends on Phase 2

```
Week 6: Translation Function
├── Implement [[T]] : Type → Relation
├── Handle Pi, Sort, App cases
├── Build relation types
└── Unit tests

Week 7: Proof Generation
├── Generate proof obligations
├── Implement logical relations
├── Build proof terms
└── Integration tests

Week 8-9: Free Theorems
├── Auto-generate free theorems
├── Example library (map, reverse, fold)
├── Browser demo with AI suggestions
└── Documentation
```

**Deliverables**:
- ✅ Parametricity module
- ✅ Free theorem generator
- ✅ Example library (10+ theorems)
- ✅ Browser demo with LLM integration

**Success Metrics**:
- <5ms translation per type
- <50ms proof generation
- 20+ free theorems generated

---

### Phase 4: Stretch Goals (Weeks 10-16) - OPTIONAL

#### Option A: π₁(S¹) ≃ ℤ (Homotopy Type Theory)

**Requires**: Type system extension (HITs, paths)

```
Weeks 10-11: Type System Extension
├── Add PathType to TermKind
├── Implement HIT support
├── Add path constructors
└── Update type checker

Weeks 12-13: Circle Type
├── Define S¹ with base and loop
├── Implement path induction
├── Build loop space
└── Unit tests

Weeks 14-15: Fundamental Group
├── Implement encode/decode
├── Prove equivalence
├── Build isomorphism
└── Integration tests

Week 16: Visualization
├── Interactive circle demo (WASM)
├── Visual path composition
├── AI-assisted topology
└── Documentation
```

#### Option B: Strong Normalization

**Requires**: Reducibility candidates

```
Weeks 10-11: Reducibility Theory
├── Define reducibility candidates
├── Implement CR1-CR3 properties
├── Build proof structure
└── Unit tests

Weeks 12-14: Proof Construction
├── Prove SN for Π-types
├── Handle universe levels
├── Prove totality
└── Integration tests

Weeks 15-16: Validation
├── Self-apply to lean-agentic
├── Prove type checker terminates
├── Generate certificate
└── Documentation
```

---

## 🎪 Feature Dependencies

```
                     ┌──────────────┐
                     │ Type System  │
                     │  (Π-types,   │
                     │  universes)  │
                     └──────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
      ┌──────────┐  ┌──────────┐  ┌──────────┐
      │ Church-  │  │Normaliz- │  │Parametri-│
      │ Rosser   │  │ation by  │  │  city    │
      │          │  │Evaluation│  │          │
      └────┬─────┘  └────┬─────┘  └────┬─────┘
           │             │             │
           └─────────────┼─────────────┘
                         │
                    ┌────▼─────┐
                    │   All    │
                    │ Theorems │
                    │  Ready   │
                    └──────────┘
```

**Key Insight**: All Phase 1-3 theorems can build in parallel after Church-Rosser!

---

## 💎 Unique Value Propositions

### For Each Theorem

| Theorem | Hash-Consing Benefit | WASM Demo | AI Integration | Research Impact |
|---------|---------------------|-----------|----------------|----------------|
| **Church-Rosser** | 150x faster equality | ✅ Visual reduction | Suggest rewrites | Foundation |
| **NbE** | O(1) term lookup | ✅ Interactive norm | Predict normal form | Performance |
| **Parametricity** | O(1) relation check | ✅ Free theorem viz | Generate proofs | Cutting-edge |
| **π₁(S¹) ≃ ℤ** | Fast path equality | ✅ 3D topology | Tactic suggestions | Prestige |
| **Strong Norm** | Cached reducibility | ✅ Termination viz | Proof automation | Ultimate |

---

## 🚀 Quick Start Guide

### Getting Started (First 30 Minutes)

```bash
# 1. Create theorem workspace
cd /workspaces/lean-agentic
cargo new lean-agentic-theorems --lib

# 2. Set up dependencies
cat >> lean-agentic-theorems/Cargo.toml <<EOF
[dependencies]
lean-agentic = { path = "../lean-agentic" }
leanr-elab = { path = "../leanr-elab" }
leanr-eval-lite = { path = "../leanr-eval-lite" }
EOF

# 3. Create module structure
mkdir -p lean-agentic-theorems/src/{confluence,nbe,parametricity}

# 4. Start with Church-Rosser
cat > lean-agentic-theorems/src/confluence/mod.rs <<'EOF'
//! Church-Rosser confluence proof for lean-agentic
//!
//! Proves that βδιζ-reduction is confluent, ensuring
//! definitional equality is well-defined.

pub mod parallel_reduction;
pub mod diamond_property;

pub use parallel_reduction::ParallelReducer;
pub use diamond_property::ConfluenceChecker;
EOF

# 5. Run first test
cargo test -p lean-agentic-theorems
```

---

## 📊 Success Criteria

### Phase 1 (Church-Rosser) ✅
- [ ] Confluence checker passes 100+ test cases
- [ ] <5ms confluence checks on typical terms
- [ ] 150x speedup demonstrated in benchmarks
- [ ] Browser demo deployed to GitHub Pages
- [ ] Documentation complete with examples

### Phase 2 (NbE) ⏳
- [ ] Normalization <1ms for typical proofs
- [ ] >95% cache hit rate in practice
- [ ] Integration tests with type checker pass
- [ ] Performance comparison shows 150x speedup
- [ ] API documentation complete

### Phase 3 (Parametricity) ⏳
- [ ] 20+ free theorems auto-generated
- [ ] <50ms proof generation per theorem
- [ ] AI suggestions working in browser demo
- [ ] Published as standalone library
- [ ] Research paper draft complete

### Phase 4 (Stretch Goals) ⏸️
- [ ] HoTT or Strong Norm implementation complete
- [ ] Novel research contribution identified
- [ ] Conference paper submitted (ICFP/POPL/CPP)
- [ ] Community adoption (>10 GitHub stars)
- [ ] Blog post published

---

## 🎯 Decision Matrix: Which Theorem to Implement First?

### Scoring (1-5 scale)

| Criterion | Church-Rosser | NbE | Parametricity | π₁(S¹)≃ℤ | Strong Norm |
|-----------|--------------|-----|---------------|----------|-------------|
| **Ease** | 🟢 5/5 | 🟡 3/5 | 🟡 2/5 | 🔴 1/5 | 🔴 1/5 |
| **Timeline** | 🟢 2 wks | 🟡 3 wks | 🟠 4 wks | 🔴 6 wks | 🔴 6 wks |
| **Immediate Value** | 🟢 5/5 | 🟢 5/5 | 🟡 3/5 | 🟠 2/5 | 🟠 2/5 |
| **Research Impact** | 🟡 3/5 | 🟢 4/5 | 🟢 5/5 | 🟢 5/5 | 🟢 5/5 |
| **Demo Value** | 🟢 4/5 | 🟢 5/5 | 🟢 4/5 | 🟢 5/5 | 🟡 3/5 |
| **Hash-Consing** | 🟢 5/5 | 🟢 5/5 | 🟢 4/5 | 🟢 4/5 | 🟢 4/5 |
| **AI Integration** | 🟡 3/5 | 🟢 4/5 | 🟢 5/5 | 🟢 4/5 | 🟢 4/5 |
| **TOTAL** | **27/35** | **29/35** | **27/35** | **26/35** | **24/35** |

### 🏆 Winner: Normalization by Evaluation (NbE)

**But recommend starting with Church-Rosser** because:
1. ✅ Validates existing implementation
2. ✅ Faster learning curve (2 weeks)
3. ✅ Foundation for other theorems
4. ✅ Immediate confidence boost

**Then implement NbE** for:
1. ⚡ Maximum performance showcase
2. 🔧 Practical utility (powers type checking)
3. 📈 Highest total score

---

## 🎓 Learning Path

### For New Contributors

```
┌─────────────┐
│ Read Source │ (2 hours)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Understand   │ (4 hours)
│Hash-Consing │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Implement   │ (1 week)
│ Church-     │
│ Rosser      │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Benchmark & │ (3 days)
│ Document    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Choose Next │
│ Theorem     │
└─────────────┘
```

---

## 📚 Resources

### Essential Reading
1. **Church-Rosser**: "Confluence for Dependent Types" - Siles et al. (2023)
2. **NbE**: "Checking Dependent Types with NbE" - Christiansen tutorial
3. **Parametricity**: "The Marriage of Univalence and Parametricity" - Tabareau (2024)
4. **HoTT**: "HoTT Book" Chapter 8 (Homotopy Theory)
5. **Strong Norm**: "Calculus of Constructions" - Coquand & Huet (1988)

### Implementation Examples
- **Lean 4**: https://github.com/leanprover/lean4
- **Agda**: https://github.com/agda/agda
- **Coq**: https://github.com/coq/coq
- **Arend**: https://github.com/JetBrains/Arend (has NbE)

---

## 🤝 Collaboration

### Suggested Agent Swarm

```
┌──────────────┐
│   Planner    │  (Coordinates phases)
│    Agent     │
└──────┬───────┘
       │
   ┌───┴────┬────────┬────────┐
   ▼        ▼        ▼        ▼
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
│Coder │ │Tester│ │Review│ │Doc   │
│Agent │ │Agent │ │Agent │ │Agent │
└──────┘ └──────┘ └──────┘ └──────┘
```

### Memory Coordination
```javascript
// Progress tracking
memory.store("theorem/church-rosser/status", {
  phase: "parallel_reduction",
  progress: 60,
  tests_passing: 18,
  blockers: []
});

// Knowledge sharing
memory.store("theorem/confluence/insight", {
  finding: "Hash-consing reduces confluence checks from O(n) to O(1)",
  benchmark: "150x speedup on typical terms",
  recommendation: "Cache all reduction paths"
});
```

---

## 🎉 Milestones

### 🏁 Milestone 1: First Theorem Complete (Week 2)
- Church-Rosser implemented
- Browser demo live
- Blog post published
- Community feedback

### 🏁 Milestone 2: Performance Showcase (Week 5)
- NbE implemented
- 150x speedup measured
- Benchmarks published
- Performance blog post

### 🏁 Milestone 3: Research Contribution (Week 9)
- Parametricity implemented
- Free theorems generated
- Research paper draft
- Conference submission

### 🏁 Milestone 4: World-Class System (Week 16)
- HoTT or Strong Norm complete
- Full theorem suite
- Research paper accepted
- lean-agentic 1.0 release

---

## 📞 Getting Help

- **Documentation**: `/workspaces/lean-agentic/docs/`
- **Examples**: `/workspaces/lean-agentic/examples/`
- **Research Report**: `THEOREM_RESEARCH_REPORT.md`
- **Implementation Guides**: `THEOREM_IMPLEMENTATION_GUIDES.md`
- **GitHub Issues**: Tag with `theorem` label

---

## ✅ Next Action Items

1. **Review this roadmap** with team
2. **Create lean-agentic-theorems workspace**
3. **Set up project structure**
4. **Begin Church-Rosser implementation**
5. **Schedule weekly progress reviews**

---

**Last Updated**: 2025-10-25
**Status**: Ready to Begin Phase 1
**Estimated Completion**: 16 weeks for full roadmap
**Minimum Viable**: 9 weeks for Phases 1-3

🚀 **Let's prove some theorems!**
