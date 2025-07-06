# Seal Web Application - Development Guide

## Project Overview

This is the web frontend for the Seal TypeScript type checker, built with React, TypeScript, and Monaco Editor. It uses WebAssembly to run the Rust-based type checker directly in the browser.

## Tech Stack

- **React 19** - UI framework
- **TypeScript** - Type safety
- **Monaco Editor** - Code editor with syntax highlighting
- **Vite** - Build tool and dev server
- **Biome** - Linting and formatting
- **Bulma** - CSS framework
- **WebAssembly** - Rust type checker integration

## Development Workflow

### ⚠️ CRITICAL: Code Quality Enforcement

**ALL changes MUST pass the `pnpm check` command before being committed.**

The `check` command runs:
- `biome check` - Linting and code style validation
- `tsc --noEmit` - TypeScript type checking

```bash
# REQUIRED before every commit
pnpm check
```

### Development Commands

```bash
# Start development server (builds WASM + starts Vite)
pnpm dev

# Build for production
pnpm build

# Run all quality checks (REQUIRED before commits)
pnpm check

# Format code
pnpm fmt

# Fix linting issues
pnpm lint

# Type check only
pnpm tsc

# Preview production build
pnpm preview
```

### Development Flow

1. **Before starting work**: Run `pnpm check` to ensure clean state
2. **During development**: Use `pnpm dev` for live reloading
3. **Before committing**: ALWAYS run `pnpm check` and fix all issues
4. **Code formatting**: Use `pnpm fmt` for consistent formatting
5. **Linting fixes**: Use `pnpm lint` for auto-fixable issues

## Project Structure

```
web/
├── src/
│   ├── App.tsx              # Main application component
│   ├── MonacoEditor.tsx     # Monaco Editor wrapper
│   ├── types.ts             # TypeScript type definitions
│   ├── main.tsx             # App entry point
│   ├── index.css            # Global styles
│   ├── App.css              # App-specific styles
│   └── wasm/                # Generated WASM files
│       ├── seal_cli.js      # WASM JavaScript bindings
│       ├── seal_cli_bg.wasm # WASM binary
│       └── *.d.ts           # TypeScript definitions
├── biome.json               # Biome configuration
├── tsconfig.json            # TypeScript configuration
├── package.json             # Dependencies and scripts
├── vite.config.ts           # Vite configuration
└── build-wasm.sh            # WASM build script
```

## Code Quality Standards

### TypeScript
- All code must be strongly typed
- No `any` types allowed
- Use interfaces for component props
- Proper error handling with type guards

### React Best Practices
- Use functional components with hooks
- Proper dependency arrays in useEffect
- Use useCallback for event handlers when needed
- No inline object/array creation in JSX props

### Code Style (Enforced by Biome)
- Tab indentation (2 spaces displayed)
- Double quotes for strings
- Template literals over string concatenation
- Optional chaining over non-null assertions
- Semantic HTML elements (button vs div for interactions)

## Error Handling Patterns

### WASM Integration
```typescript
try {
  const wasm = await import("./wasm/seal_cli.js")
  await wasm.default()
  setWasmModule(wasm as unknown as WasmModule)
} catch (error) {
  console.error("Failed to load WASM module:", error)
  // Graceful fallback
}
```

### Monaco Editor Integration
```typescript
const updateEditorMarkers = useCallback((errors: TypeCheckError[]) => {
  if (!monacoRef.current || !editorRef.current) return
  
  // Safe access with optional chaining
  monacoRef.current?.editor.setModelMarkers(
    model,
    "seal-type-checker", 
    markers
  )
}, [])
```

### Type Checking Error Display
```typescript
const handleErrorClick = (error: TypeCheckError) => {
  if (!editorRef.current) return
  
  editorRef.current.setPosition({
    lineNumber: error.start_line,
    column: error.start_column,
  })
}
```

## Build Process

### WASM Generation
The `build-wasm.sh` script:
1. Builds the Rust code with `wasm-pack`
2. Generates JavaScript bindings
3. Copies files to `src/wasm/` directory

### Vite Build
- TypeScript compilation
- React component bundling
- Asset optimization
- Monaco Editor worker setup

## Testing Strategy

### Manual Testing Checklist
- [ ] Type checker loads without errors
- [ ] Code editing works smoothly
- [ ] Type errors appear correctly positioned
- [ ] Error clicking navigates to correct location
- [ ] WASM module loads successfully
- [ ] No console errors in browser

### Automated Quality Checks
- **TypeScript**: `tsc --noEmit` catches type errors
- **Biome**: Catches code style and logic issues
- **Build**: `pnpm build` validates production readiness

## Common Issues & Solutions

### WASM Loading Fails
- Check if `build-wasm.sh` ran successfully
- Verify WASM files exist in `src/wasm/`
- Check browser console for detailed errors

### Type Errors
- Run `pnpm tsc` for detailed TypeScript errors
- Check import paths and type definitions
- Verify Monaco Editor types are properly imported

### Biome Warnings
- Run `pnpm lint` to auto-fix issues
- For non-null assertions: Use optional chaining
- For array keys: Use stable identifiers instead of index

## Integration with Main Project

### WASM Interface
The web app expects these WASM exports:
```typescript
interface WasmModule {
  type_check: (code: string) => TypeCheckResult
  default: () => Promise<void>
}

interface TypeCheckResult {
  errors: TypeCheckError[]
}

interface TypeCheckError {
  message: string
  start_line: number
  start_column: number
  end_line?: number
  end_column?: number
}
```

### Development with Main Crate
1. Make changes to Rust code in `crates/`
2. Run `./build-wasm.sh` to regenerate WASM
3. Test in web interface with `pnpm dev`
4. Always run `pnpm check` before committing

## Performance Considerations

### Monaco Editor
- Web workers are configured for TypeScript/JSON
- Language services are partially disabled to avoid conflicts
- Editor is configured for optimal TypeScript editing

### WASM Loading
- WASM module is loaded asynchronously
- Loading state is shown to user
- Graceful error handling for load failures

### Type Checking
- Debounced type checking (300ms delay)
- Errors are cached and updated incrementally
- Editor markers are updated efficiently

---

**Remember: Always run `pnpm check` before committing. This ensures code quality and prevents build failures.**