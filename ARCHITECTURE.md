# Seal Architecture

This document describes the high-level architecture and design decisions of the Seal TypeScript type checker.

## Overview

Seal is designed as a modular type checker that processes TypeScript/JavaScript code through several distinct phases:

```
Source Code → SWC Parser → SWC AST → SIR → Type Checker → Type Errors
```

## Core Components

### 1. Parser (SWC)

We use the SWC parser for parsing TypeScript/JavaScript source code. This provides:
- Fast, production-ready parsing
- Full TypeScript syntax support
- Accurate source location information
- Active maintenance and community support

### 2. Seal Intermediate Representation (SIR)

The SIR (implemented in the `seal-ty` crate as a module) is our custom intermediate representation designed specifically for type checking. Key design decisions:

- **Separation of Concerns**: SIR separates parsing concerns from type checking logic
- **Simplified AST**: Removes syntax sugar and normalizes similar constructs
- **Type-Centric**: Designed to make type analysis straightforward
- **Source Mapping**: Maintains accurate source locations for error reporting

#### SIR Structure

```rust
Program
├── Items (declarations at file level)
│   ├── Import/Export
│   ├── TypeAlias
│   ├── Interface
│   └── Statement
└── Statements
    ├── VariableDeclaration
    ├── FunctionDeclaration
    ├── ClassDeclaration
    ├── Block
    ├── Return
    ├── If/Else
    └── Expression
```

### 3. Type System (`seal-ty`)

The `seal-ty` crate is the core of Seal, containing:
- The type system implementation
- SIR (Seal Intermediate Representation) module
- Type checker
- Parser integration with SWC
- Context and symbol management
- Type interning for efficiency

The type system is built around a core `Ty` type that represents all possible types:

```rust
// Types are represented through TyKind enum and managed via Ty handles
pub enum TyKind {
    // Primitive types
    Any,
    Boolean,
    Number,
    String,
    // Complex types
    Object { ... },
    Function { ... },
    Class { ... },
    Interface { ... },
    Union(...),
    Intersection(...),
    // Special types
    Unknown,
    Never,
    Error,
}
```

#### Type Checking Context

The `Context` struct maintains:
- Variable bindings and their types
- Type definitions (interfaces, type aliases)
- Scope management
- Error accumulation

### 4. Parser Integration

The `seal-ty` crate directly integrates with the SWC parser to:
- Parse TypeScript/JavaScript source code
- Transform SWC AST nodes into internal representations
- Handle syntactic desugaring
- Preserve source locations for error reporting

## Type Checking Algorithm

### 1. Two-Pass Analysis

**First Pass**: Collect all type declarations
- Interfaces
- Type aliases
- Class declarations
- Function signatures

**Second Pass**: Type check implementations
- Variable initializations
- Function bodies
- Expression types
- Statement flow

### 2. Type Satisfaction

The core type checking operation is "satisfaction" - determining if type A satisfies type B:

```rust
fn satisfies(given: &Type, expected: &Type) -> bool
```

Rules:
- Any type satisfies `any`
- Same types satisfy each other
- Union types: at least one member must satisfy
- Object types: structural typing (all expected properties must exist and satisfy)

### 3. Type Inference

Currently limited to:
- Literal expressions (infer from value)
- Variable declarations with initializers
- Function return types from return statements

## Error Handling

Errors are collected during type checking and include:
- Error kind (type mismatch, undefined variable, etc.)
- Source location (file, line, column)
- Expected vs actual types
- Contextual information

## Future Architecture Considerations

### 1. Incremental Type Checking
- Cache type information per module
- Re-check only changed files and dependencies
- Maintain dependency graph

### 2. Language Server Protocol (LSP)
- Real-time type checking
- IDE integration
- Hover information and diagnostics

### 3. Type Inference Engine
- Constraint-based type inference
- Generic type parameter inference
- Flow-sensitive typing

### 4. Module System
- Module resolution algorithm
- Declaration file (.d.ts) support
- Package.json exports handling

## Performance Considerations

1. **Memory Efficiency**
   - Intern common types (primitives)
   - Share type definitions across scopes
   - Lazy evaluation where possible

2. **Parallel Processing**
   - Type check independent modules in parallel
   - Concurrent error collection
   - Parallel SIR generation

3. **Caching**
   - Cache parsed ASTs
   - Cache type check results
   - Incremental updates

## Testing Strategy

1. **Unit Tests**: Test individual type checking rules
2. **Integration Tests**: Test complete programs
3. **Conformance Tests**: TypeScript official test suite (future)
4. **Performance Tests**: Benchmark against large codebases

## Code Organization

```
seal/
├── crates/
│   └── seal-ty/         # Core type checker crate
│       ├── src/
│       │   ├── checker/     # Type checking logic
│       │   ├── context.rs   # Type checking context
│       │   ├── intern/      # Type interning
│       │   ├── kind/        # Type kinds
│       │   ├── parse.rs     # Parser integration
│       │   ├── sir.rs       # Seal Intermediate Representation
│       │   ├── symbol.rs    # Symbol handling
│       │   └── ty.rs        # Type system core
│       └── tests/       # Unit and integration tests
├── tests/               # Integration tests (future)
└── benches/            # Performance benchmarks (future)
```

The current architecture uses a monolithic approach with all functionality in the `seal-ty` crate. This provides:
- Simpler initial development
- Easier refactoring as the design evolves
- Direct access between components
- Potential for future modularization as needs become clearer