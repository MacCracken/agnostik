# Benchmarks — Rust vs Cyrius (legacy reference)

> **Legacy / historical.** Captured at the Rust → Cyrius port boundary
> (Cyrius `v0.95.0` / `cc2 v2.6.4` vs Rust `v0.90.0` / Criterion) to
> frame the migration cost-benefit at the time. **Numbers below are
> not current** — the Cyrius side has moved through several toolchain
> revs (cc2 → cc5; pinned at 5.7.12 as of 1.0.0) and agnostik through
> the 1.0.0 audit (CSPRNG migration alone moved `agent_id_new` from
> ~35 ns to ~600 ns; see `CHANGELOG.md` 1.0.0 / F-001).
>
> Live benchmark data lives at [`history.csv`](history.csv); current
> baseline runs via `cyrius bench tests/bcyr/agnostik.bcyr`. This
> file is preserved only to document the Rust comparison that
> motivated the port.

Cyrius v0.95.0 (cc2 v2.6.4) vs Rust v0.90.0 (Criterion) — comparable operations measured on the same hardware.

## Head-to-Head Comparison

| Operation | Rust (Criterion) | Cyrius (bench.bcyr) | Delta |
|-----------|-----------------|---------------------|-------|
| `agent_id_new` | 48 ns | 35 ns | **1.4x faster** |
| `trace_context_new` | 93 ns | 97 ns | 1.04x slower |
| `trace_context_child` | 52 ns | 41 ns | **1.3x faster** |
| `sandbox_config_default` | 27 ns | 62 ns | 2.3x slower |
| `agent_id_to_str` | — | 220 ns | — |
| `agent_id_roundtrip` | 91 ns (ser+deser) | 530 ns | 5.8x slower |
| `version_to_str` | — | 149 ns | — |
| `version_roundtrip` | — | 278 ns | — |
| `traceparent_format` | — | 826 ns | — |
| `trace_context_serialize` | 167 ns | — | — |
| `trace_context_deserialize` | 317 ns | — | — |
| `sandbox_config_serialize` | 232 ns | — | — |
| `sandbox_config_deserialize` | 323 ns | — | — |
| `inference_request_full` | — | 519 ns | — |
| `audit_entry_full` | — | 812 ns | — |
| `accelerator_device_full` | — | 153 ns | — |
| `security_context_full` | — | 232 ns | — |
| `message_build_3turn` | — | 435 ns | — |
| `token_usage_update` | — | 36 ns | — |

### Analysis

- **agent_id_new** (1.4x faster) — Cyrius uses xorshift64 PRNG seeded from `/dev/urandom` on first call. Rust uses `uuid::Uuid::new_v4()` which calls `getrandom` per invocation.
- **trace_context_child** (1.3x faster) — Cyrius generates child span inline with single xorshift call. Rust clones struct + generates new UUID.
- **sandbox_config_default** (2.3x slower) — Rust's `Default::default()` is zero-cost stack initialization. Cyrius heap-allocates via `alloc()` + stores 6 fields. Expected for a bump-allocator model.
- **agent_id_roundtrip** (5.8x slower) — Rust serde is optimized by LLVM (inlined serializer, zero-copy deserializer). Cyrius does manual hex parsing byte-by-byte. Functionally correct, no LLVM optimization passes.
- **trace_context_new** (~parity) — Both do similar work: generate trace_id + span_id + set flags.

### Where Cyrius wins

- agent_id_new: 35ns vs 48ns (PRNG vs syscall)
- trace_context_child: 41ns vs 52ns (inline generation)
- Zero startup time (no runtime, no allocator init beyond brk)
- 148 KB test binary vs 8.7 MB Rust release binary (59x smaller)
- Zero external dependencies vs 25+ crates
- Compile time: ~200ms vs ~15s

### Where Rust wins

- sandbox_config_default: 27ns vs 62ns (stack vs heap)
- Serde roundtrips: LLVM-optimized serialize/deserialize
- SIMD-friendly string operations
- Stack allocation for small structs (no heap pressure)

## Full Cyrius Benchmark Results (v0.95.0)

| Benchmark | Avg | Min | Max | Iterations |
|-----------|-----|-----|-----|-----------|
| `agent_id_new` | 35 ns | 28 ns | 45 ns | 10,000 |
| `trace_context_new` | 97 ns | 86 ns | 150 ns | 10,000 |
| `trace_context_child` | 41 ns | 37 ns | 53 ns | 10,000 |
| `sandbox_config_default` | 62 ns | 55 ns | 68 ns | 10,000 |
| `agent_id_to_str` | 220 ns | 204 ns | 265 ns | 10,000 |
| `agent_id_roundtrip` | 530 ns | 487 ns | 634 ns | 10,000 |
| `version_to_str` | 149 ns | 142 ns | 207 ns | 10,000 |
| `version_roundtrip` | 278 ns | 264 ns | 370 ns | 10,000 |
| `traceparent_format` | 826 ns | 802 ns | 871 ns | 5,000 |
| `inference_request_full` | 519 ns | 481 ns | 785 ns | 5,000 |
| `audit_entry_full` | 812 ns | 774 ns | 1 us | 5,000 |
| `accelerator_device_full` | 153 ns | 145 ns | 162 ns | 5,000 |
| `security_context_full` | 232 ns | 210 ns | 467 ns | 5,000 |
| `message_build_3turn` | 435 ns | 409 ns | 441 ns | 5,000 |
| `token_usage_update` | 36 ns | 34 ns | 41 ns | 10,000 |

## Rust v0.90.0 Reference (from bench-history.csv, last 4 runs)

| Benchmark | Run 1 | Run 2 | Run 3 | Run 4 |
|-----------|-------|-------|-------|-------|
| `agent_id_new` | 48 ns | 47 ns | 48 ns | 46 ns |
| `trace_context_new` | 92 ns | 94 ns | 94 ns | 93 ns |
| `trace_context_child` | 48 ns | 52 ns | 52 ns | 52 ns |
| `sandbox_config_default` | 26 ns | 27 ns | 27 ns | 26 ns |
| `trace_context_serialize` | — | — | 168 ns | 167 ns |
| `trace_context_deserialize` | — | — | 325 ns | 315 ns |
| `sandbox_config_serialize` | — | — | 262 ns | 225 ns |
| `sandbox_config_deserialize` | — | — | 331 ns | 317 ns |
| `agent_id_serialize` | — | — | 33 ns | 33 ns |
| `agent_id_deserialize` | — | — | 59 ns | 59 ns |

## Migration Summary

| Metric | Rust v0.90.0 | Cyrius v0.95.0 |
|--------|-------------|----------------|
| Source lines | 7,121 | ~3,200 |
| Binary size | 8.7 MB (.rlib) | 148 KB (test ELF) |
| Dependencies | 25 crates | 0 |
| Compile time | ~15s | ~200ms |
| Tests | 107 | 226 |
| Benchmarks | 10 | 15 |

Run benchmarks: `cyrius bench`
