# ALICE Compiler SaaS

DSL compiler platform powered by the ALICE ecosystem. Compile, run, parse, and optimize domain-specific language source code via a simple REST API.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

## Status

| Check | Status |
|-------|--------|
| `cargo check` | passing |
| API health | `/health` |

## Quick Start

```bash
docker compose up -d
```

API Gateway: http://localhost:8080
Compiler Engine: http://localhost:8129

## Architecture

```
Client
  |
  v
API Gateway     :8080
  |
  v
Compiler Engine :8129
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/v1/compiler/compile` | Compile DSL source to bytecode |
| `POST` | `/api/v1/compiler/run` | Execute compiled bytecode |
| `POST` | `/api/v1/compiler/parse` | Parse source and return AST metadata |
| `POST` | `/api/v1/compiler/optimize` | Optimize bytecode with configurable passes |
| `GET` | `/api/v1/compiler/stats` | Compilation statistics |
| `GET` | `/health` | Service health check |

### compile

```json
POST /api/v1/compiler/compile
{
  "source": "fn main() { let x = 1 + 2; }",
  "language": "alice-dsl",
  "target": "wasm32",
  "optimize": true
}
```

Response:
```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "success",
  "language": "alice-dsl",
  "target": "wasm32",
  "bytecode_size_bytes": 72,
  "symbol_count": 1,
  "warnings": [],
  "errors": [],
  "elapsed_us": 42
}
```

### run

```json
POST /api/v1/compiler/run
{
  "bytecode": "<base64>",
  "args": ["--verbose"],
  "timeout_ms": 5000
}
```

### parse

```json
POST /api/v1/compiler/parse
{
  "source": "fn add(a: i32, b: i32) -> i32 { a + b }",
  "language": "alice-dsl"
}
```

### optimize

```json
POST /api/v1/compiler/optimize
{
  "bytecode": "<base64>",
  "level": 2,
  "passes": ["dead-code-elimination", "inline-expansion"]
}
```

## Supported Targets

| Target | Description |
|--------|-------------|
| `wasm32` | WebAssembly 32-bit |
| `wasm64` | WebAssembly 64-bit |
| `x86_64` | x86-64 native |
| `aarch64` | ARM64 native |
| `llvm-ir` | LLVM IR text |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `COMPILER_ADDR` | `0.0.0.0:8129` | Compiler engine bind address |
| `CORE_ENGINE_URL` | `http://core-engine:8129` | Core engine URL for gateway |
| `JWT_SECRET` | `dev-secret-change-me` | JWT signing secret |

## License

AGPL-3.0. Commercial dual-license available — contact for pricing.
