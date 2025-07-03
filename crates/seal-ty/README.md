# seal-ty

Core type checking crate for the Seal TypeScript type checker.

## Overview

This crate implements the complete type checking functionality for TypeScript/JavaScript, including:

- Type system with support for primitives, objects, functions, classes, interfaces, and unions
- Seal Intermediate Representation (SIR) for type analysis
- Integration with SWC parser for TypeScript/JavaScript parsing
- Type satisfaction checking and basic type inference
- Error reporting with source locations

## Architecture

### Key Modules

- **`checker/`** - Type checking implementation
  - `base/` - Core checking logic for expressions, statements, declarations
  - `class.rs` - Class type checking
  - `function.rs` - Function type checking
  - `errors.rs` - Error types and reporting

- **`sir.rs`** - Seal Intermediate Representation
  - Simplified AST designed for type checking
  - Maintains source location information

- **`parse.rs`** - SWC parser integration
  - Handles TypeScript/JavaScript parsing
  - Converts SWC AST to internal representations

- **`context.rs`** - Type checking context
  - Variable bindings and scopes
  - Type definitions storage
  - Error accumulation

- **`ty.rs`** - Core type system
  - Type definitions and operations
  - Type equality and satisfaction

- **`intern/`** - Type interning
  - Memory-efficient type storage
  - Type deduplication

## Usage

This crate is designed to be used as a library. Example usage:

```rust
use seal_ty::{check_program, parse};

let source = "const x: number = 42;";
let program = parse(source)?;
let errors = check_program(&program);

if errors.is_empty() {
    println!("Type check passed!");
} else {
    for error in errors {
        println!("Error: {:?}", error);
    }
}
```

## Current Limitations

Many TypeScript features are not yet implemented:
- Array and tuple types
- Generic types and type parameters
- Full module system support
- Many expression and statement types
- Advanced type features (mapped types, conditional types, etc.)

See the main project README for a complete list of supported and unsupported features.

## Development

When adding new features:

1. Update the SIR if new AST nodes are needed
2. Add type checking logic in the appropriate checker module
3. Add tests to verify the implementation
4. Update error types if new error cases are introduced

See CONTRIBUTING.md in the project root for detailed guidelines.