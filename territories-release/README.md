# Territories

Competitive 15×40 territorial board game built with React, Vite, and Rust/WebAssembly.

## Quickstart

```bash
npm install
npm run dev
```

## Production Build & Preview

```bash
npm run build
npm run preview
```

## Rebuilding WebAssembly Core

Prebuilt WASM bindings live in `src/pkg/` so Node-only environments do not require Rust. If you modify `territories-wasm/`, rebuild them with:

```bash
npm run build:wasm
```
