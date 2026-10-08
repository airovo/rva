# Security Policy

## Supported versions

RVA is pre-1.0 (experimental). Security fixes are applied to the latest `main`.

## Reporting a vulnerability

Please **do not** open a public issue for security problems. Report privately to
**security@airovo.tech** with:

- a description of the issue and its impact,
- a minimal reproducer (asset, viewport, runtime),
- the affected component (core / cli / wasm / ffi / server / adapter) and version.

We aim to acknowledge reports within a few business days and will coordinate a
disclosure timeline with you.

## Scope notes

- RVA is a data format, not an execution environment. An `.rva` asset must never
  be able to run code, navigate, or perform network I/O merely by being rendered.
- The primary attack surface is the parser/resolver (untrusted `.rva` input) and
  the native FFI boundary. The `corpus/` fuzzing harness (`core/tests/fuzz.rs`)
  exists to harden the parser against malformed input.
- CTA Regions are semantic geometry only; behavior is host-owned and outside the
  asset.
