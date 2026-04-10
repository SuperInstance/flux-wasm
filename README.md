# flux-wasm

FLUX bytecode VM compiled to WebAssembly via Rust + wasm-bindgen.

Run FLUX bytecode in any browser or WASM runtime.

## Features
- Full FLUX VM interpreter (30+ opcodes)
- Two-pass assembler (text to bytecode)
- Markdown/natural language to bytecode converter
- A2A agent messaging via JS interop

## Building

```bash
wasm-pack build --target web
```

## Part of the FLUX Ecosystem
- [flux-runtime](https://github.com/SuperInstance/flux-runtime) — Python (1944 tests)
- [flux-core](https://github.com/SuperInstance/flux-core) — Rust
- [flux-zig](https://github.com/SuperInstance/flux-zig) — Fastest VM (210ns/iter)
- [flux-benchmarks](https://github.com/SuperInstance/flux-benchmarks) — Performance data
