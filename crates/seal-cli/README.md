# seal-cli

A command-line interface for the Seal TypeScript type checker.

## Installation

From the project root:

```bash
cargo build --bin seal
```

## Usage

### Type Check a File

```bash
cargo run --bin seal check path/to/file.ts
```

Or if you have the binary installed:

```bash
seal check path/to/file.ts
```

### Examples

Check a TypeScript file:
```bash
cargo run --bin seal check examples/hello.ts
```

The CLI will:
- Parse the TypeScript file
- Run the Seal type checker
- Display type errors if any are found
- Exit with code 0 on success, 1 on failure

## Commands

- `check <file>` - Type check a TypeScript file

## Exit Codes

- `0` - Type checking passed
- `1` - Type checking failed or other error occurred