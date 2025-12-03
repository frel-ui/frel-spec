Approach: Semantic Analysis → IR → Codegen (in that order) Why this order:

1. Semantic analysis provides the information IR needs (resolved types, scopes, dependencies)
2. IR design is informed by what semantic analysis can provide
3. IR becomes the host-agnostic contract that all plugins consume

**Four main phases:**

1. Semantic Foundation - Diagnostics, symbol tables, name resolution, type resolution
2. IR Design - Blueprint IR (closure schemes, bindings, 3.call sites), Expression IR, dependency analysis
3. AST → IR Builder - Transform analyzed AST to IR with synthetic field generation
4. Codegen Migration - Plugins consume IR instead of AST

The most critical IR structure is BlueprintIR which must capture:

- Closure scheme (params + locals + synthetic fields)
- Internal bindings with selectors
- Call sites with subscription wiring
- Control flow (when/repeat/select)