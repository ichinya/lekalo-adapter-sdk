# lekalo-adapter-sdk

Optional SDKs for independently authored Lekalo adapters. The host does not link this repository or register its implementation languages.

The Go, PHP, Rust and TypeScript packages, generator, reference templates and 1,149-vector corpus live here. Their existing `lekalo.target/v1` API and versions are preserved. The newer `lekalo/runtime/v1` contract is pinned in `contracts/external-runtime.schema.v1.json`; these legacy SDK helpers must not be presented as an implementation of that different protocol.

## Develop and verify

- `node scripts/generate-adapter-sdk.mjs --check` checks reproducible generation for all four packages.
- `node --test sdk/typescript/test/stream.mjs sdk/typescript/test/conformance.mjs`
- `node sdk/typescript/test/vectors.mjs` runs the TypeScript codec corpus without a host checkout.
- `cargo test --locked --manifest-path sdk/rust/Cargo.toml`
- `cd sdk/go && go test ./...`
- `node scripts/test-adapter-sdk-contracts.mjs` requires Ajv 8.20.0 in `LEKALO_AJV_NODE_PATH`.
- `node scripts/test-adapter-sdk-packages.mjs --verify --language typescript` verifies a detached packed consumer. `--language all` additionally requires Rust, Go and PHP >=8.3.

Host integration takes independently built executables, never imports host sources or builds a presumed sibling checkout:

```sh
LEKALO_BIN=/absolute/path/lekalo LEKALO_SDK_ORACLE=/absolute/path/adapter-sdk-oracle node scripts/test-adapter-sdk-vectors.mjs --language all
LEKALO_BIN=/absolute/path/lekalo node scripts/test-adapter-sdk-conformance.mjs --language all
```

`protocol-inputs.lock.json` records the exact host commit and hashes of the public schemas and handwritten test controls. These are reviewed protocol inputs, not runtime implementation dependencies. Update those pins deliberately when adopting a host contract change.

See [SDK guide](sdk/README.md), [specification](sdk/spec.md) and [generation inputs](sdk/schema-inputs.json).
