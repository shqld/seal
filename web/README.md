# Seal Web - TypeScript Type Checker in the Browser

A web interface for the Seal TypeScript type checker, running entirely in your browser via WebAssembly.

## Prerequisites

1. Install wasm-pack:
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

2. Install Node.js dependencies:
```bash
npm install
```

## Building

1. Build the WASM module:
```bash
./build-wasm.sh
# or
npm run build:wasm
```

2. Start the development server:
```bash
npm run dev
```

3. Open http://localhost:5173 in your browser

## Features

- **Monaco Editor Integration**: Full-featured code editor with TypeScript syntax highlighting
- **Real-time Type Checking**: Instant feedback as you type using Seal type checker
- **Error Markers**: Visual error indicators directly in the editor
- **Error Navigation**: Click on errors to jump to the exact location in code
- **TypeScript Language Support**: Complete TypeScript syntax and IntelliSense
- **Dark Theme**: Professional dark editor theme for comfortable coding
- **No Server Required**: Runs entirely in the browser via WebAssembly

## Development

The application consists of:
- React frontend with Bulma CSS framework
- Monaco Editor for rich code editing experience
- Seal type checker (Rust) compiled to WebAssembly via seal-cli
- Vite for development and building

## Architecture

```
Web App (React + Monaco Editor)
    ↓ (WebAssembly)
seal-cli (WASM interface)
    ↓ (Rust library)
seal-ty (TypeScript type checker)
```