# Contributing to Seal

Thank you for your interest in contributing to Seal! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Git
- Basic understanding of TypeScript type systems

### Setting Up Development Environment

1. Fork and clone the repository:
```bash
git clone https://github.com/YOUR-USERNAME/seal.git
cd seal
```

2. Build the project:
```bash
cargo build
```

3. Run tests to ensure everything is working:
```bash
cargo test
```

## Development Workflow

### Project Structure

The project consists of a single crate `seal-ty` that contains all type checking functionality:

- `src/checker/` - Type checking logic for different constructs
- `src/sir.rs` - Seal Intermediate Representation
- `src/parse.rs` - Parser integration with SWC
- `src/context.rs` - Type checking context management
- `src/ty.rs` & `src/kind/` - Core type system
- `src/intern/` - Type interning for efficiency
- `tests/` - Test suite

### Making Changes

1. Create a new branch for your feature or fix:
```bash
git checkout -b feature/your-feature-name
```

2. Make your changes following the code style guidelines

3. Add tests for your changes

4. Ensure all tests pass:
```bash
cargo test
```

5. Run the formatter and linter:
```bash
cargo fmt
cargo clippy
```

## Code Style Guidelines

### Rust Code

- Follow standard Rust naming conventions
- Use `rustfmt` for formatting
- Address all `clippy` warnings
- Prefer explicit types in function signatures
- Document public APIs with doc comments

### Error Handling

- Use `Result` types for operations that can fail
- Provide meaningful error messages
- Include source location information in type errors
- Avoid `panic!` except for truly unrecoverable errors

### Testing

- Write unit tests for new functionality
- Add integration tests for complete features
- Test both success and error cases
- Use descriptive test names

Example test structure:
```rust
#[test]
fn test_type_check_number_literal() {
    let result = check("42");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().ty, Ty::number());
}
```

## Adding New Features

### Type System Extensions

When adding new types:

1. Add the type variant to `TyKind` enum
2. Implement satisfaction rules in `checker/base/satisfies.rs`
3. Add parsing support if needed
4. Include comprehensive tests

### Expression/Statement Support

To add support for new expressions or statements:

1. Update the SIR definitions in `sir.rs`
2. Add conversion logic from SWC AST
3. Implement type checking in appropriate checker module
4. Add test cases

### Common Tasks

#### Adding a new expression type

1. Define in `sir.rs`:
```rust
pub enum Expr {
    // ... existing variants
    YourNewExpr { /* fields */ },
}
```

2. Implement checking in `checker/base/expr.rs`:
```rust
match expr {
    // ... existing cases
    Expr::YourNewExpr { .. } => {
        // Type checking logic
    }
}
```

#### Implementing a missing TypeScript feature

1. Check existing `todo!()` macros in the codebase
2. Replace with proper implementation
3. Add tests covering the feature
4. Update README.md if it's a major feature

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests

Tests are located in `crates/seal-ty/tests/`. Each test file focuses on a specific aspect:

- `check.rs` - General type checking tests
- `satisfies.rs` - Type satisfaction tests
- `build.rs` - Build and integration tests

## Debugging Tips

### Understanding Type Checking Flow

1. Enable debug logging (when implemented)
2. Use `dbg!()` macro for quick debugging
3. Check the Context state at key points
4. Trace through the SIR conversion

### Common Issues

- **Type not found**: Check if type is properly registered in context
- **Unexpected type**: Verify SIR conversion is correct
- **Missing implementation**: Look for `todo!()` in the code

## Submitting Changes

### Pull Request Process

1. Ensure your branch is up to date with main
2. Write a clear PR description explaining:
   - What changes you made
   - Why they were needed
   - Any breaking changes
3. Link related issues
4. Ensure CI passes

### PR Title Format

Use conventional commit format:
- `feat: Add array type support`
- `fix: Correct union type satisfaction`
- `docs: Update architecture documentation`
- `test: Add tests for template literals`
- `refactor: Simplify type checking context`

## Getting Help

- Open an issue for bugs or feature requests
- Ask questions in discussions
- Review existing issues and PRs for context

## Future Contributions Needed

High-priority areas needing contributions:

1. **Type System**
   - Array and tuple types
   - Generic types
   - Mapped types
   - Conditional types

2. **Language Features**
   - Module system (imports/exports)
   - Async/await
   - Decorators
   - JSX

3. **Infrastructure**
   - CLI interface
   - Error recovery
   - Performance optimizations
   - IDE integration

4. **Testing**
   - TypeScript conformance tests
   - Performance benchmarks
   - Fuzzing

Thank you for contributing to Seal!