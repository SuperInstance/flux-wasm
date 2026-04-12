# FLUX Runtime — WebAssembly Target

Browser-based FLUX VM for running agent bytecode in web contexts.

## Why WASM?

Agents shouldn't need Python or Node.js. With WASM, any browser becomes a FLUX runtime — enabling:
- Browser-based agent dashboards
- Edge deployment via CDN
- Mobile agent execution
- Plugin systems in web apps

## ISA v3 Support

All 7 ISA v3 opcodes implemented:
- **EVOLVE** (0x7C) — deterministic evolution with generation tracking
- **INSTINCT** (0x7D) — behavioral reflex stubs
- **WITNESS** (0x7E) — structured logging with PC/register/cycle
- **CONF** (0x3D) — confidence attachment
- **MERGE** (0x3E) — confidence-weighted register merge
- **SNAPSHOT** (0x7F) — state save (stub)
- **RESTORE** (0x3F) — state restore (stub)

## Build

```bash
npm install
npm run build
```

## Usage

```javascript
const fs = require('fs');
const { FluxVM } = require('./build/flux.js');

const bytecode = new Uint8Array([0x2B, 0x01, 0x0A, 0x00, 0x80]); // MOVI R1, 10; HALT
const vm = new FluxVM(bytecode);
vm.execute();
console.log('R1 =', vm.readGP(1)); // 10
```

## Cross-Language Conformance

This WASM runtime produces identical results to the Python, C, Go, Rust, and Zig runtimes when given the same bytecode input. Verified through the 88 conformance test vectors in `SuperInstance/flux-conformance`.
