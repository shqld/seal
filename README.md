# Seal

A TypeScript type checker written in Rust, leveraging the SWC parser for high-performance static analysis.

## Overview

Seal is an experimental TypeScript type checker implementation that aims to provide fast and accurate type checking for TypeScript and JavaScript codebases. Built with Rust for performance and reliability, it uses the SWC parser to analyze TypeScript/JavaScript code and performs type checking through a custom intermediate representation (SIR - Seal Intermediate Representation).

## Project Status

⚠️ **Early Development Stage** - This project is under active development and is not yet ready for production use. Many TypeScript features are still being implemented.

### Currently Implemented
- Basic type system (Boolean, Number, String, Object, Function, Class, Interface)
- Variable declarations (let, const, var) with type annotations
- Function declarations and expressions with parameter type checking
- Basic control flow (if statements, blocks, return statements)
- Union types
- Basic expression type checking (literals, identifiers, binary operations)
- Type satisfaction checking (`satisfies` operator)

### Not Yet Implemented
- Arrays and tuple types
- Generic types and type parameters
- Module system (imports/exports)
- Loops (for, while, do-while)
- Switch statements and try-catch blocks
- Template literals and JSX
- Async/await
- Type inference for complex expressions
- And many more TypeScript features...

## Architecture

The project currently consists of a single crate:

- `seal-ty` - Core type checking library containing:
  - Type system implementation
  - Seal Intermediate Representation (SIR)
  - SWC parser integration
  - Type checking logic for TypeScript/JavaScript

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Cargo

### Building
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Basic Usage
```bash
# Type check a TypeScript file (when implemented)
cargo run -- check path/to/file.ts
```

## Development

The type checker works in several phases:

1. **Parsing** - Uses SWC to parse TypeScript/JavaScript into an AST
2. **Conversion** - Transforms SWC AST into Seal IR (SIR)
3. **Type Checking** - Analyzes SIR to perform type checking
4. **Error Reporting** - Reports type errors with source locations

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design information.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

[License information to be added]

## Acknowledgments

- Built on top of the excellent [SWC](https://swc.rs/) parser
- Inspired by the TypeScript compiler and other type checking projects