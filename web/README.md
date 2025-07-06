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

- Real-time TypeScript type checking in the browser
- Error highlighting with line numbers
- Powered by Seal type checker (written in Rust)
- No server required - runs entirely via WebAssembly

## Development

The application consists of:
- React frontend with Bulma CSS framework
- Seal type checker compiled to WebAssembly
- Vite for development and building