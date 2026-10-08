## What changed

<!-- A concise description of the change and the motivation. -->

## Why

<!-- The problem this solves. Link any issue: "Closes #123". -->

## Area

- [ ] `core` (model / resolver / renderer)
- [ ] `cli` / `server` / `wasm` / `ffi`
- [ ] `adapters/*`
- [ ] `spec` / docs
- [ ] `conformance` / `corpus`

## Checks

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo deny check` (licenses/advisories/sources)
- [ ] `bash conformance/run.sh` (if core/resolver or serialization changed)
- [ ] Added a changeset (`npm run changeset`) if a published package changed

## Notes for reviewers

<!-- Anything non-obvious: trade-offs, follow-ups, screenshots. -->
