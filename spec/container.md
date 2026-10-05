# RVA Container — Draft 0.0.x (experimental)

Status: **experimental, not frozen.** Describes the container exactly as
implemented today. The signature, media type and layout may change before a
public 0.1.

## Identification

| Property | Value |
|---|---|
| Extension | `.rva` |
| Provisional media type | `application/x-rva` |
| Target registered media type | `image/rva` |
| Container | binary, self-contained |
| Signature (provisional) | magic bytes `RVA1`; final signature reserved |
| Container major version | `1` |
| Container minor version | `1` |
| Manifest `containerVersion` | `1.1` |

## Layout (little-endian)

| offset | size | field |
|---:|---:|---|
| 0 | 4 | magic / provisional signature (`RVA1`) |
| 4 | 2 | container major version |
| 6 | 2 | container minor version |
| 8 | 4 | flags |
| 12 | 8 | reserved signature bytes (currently `0`) |
| 20 | 4 | manifest length (stored bytes, `u32`) |
| 24 | 8 | resource data length (stored bytes, `u64`) |
| 32 | ... | manifest JSON (UTF-8) |
| ... | ... | concatenated resource blobs |

### Flags

| bit | meaning |
|---:|---|
| 0 | manifest region is zlib-deflated |
| 1 | resource data region is zlib-deflated |

Readers that ignore `flags` see only raw containers; readers must honor them.

## Manifest (JSON)

```jsonc
{
  "format": "RVA",
  "formatVersion": "0.1-draft",     // scene schema version
  "containerVersion": "1.1",
  "mediaType": "application/x-rva",
  "signature": "reserved; ...",
  "scene": { /* Scene model: resources, elements, topologies, text, fallback */ },
  "blobs": [
    { "id": "bgLandscape", "path": "02_backgrounds/bg-landscape.jpg",
      "offset": 0, "length": 68966, "sha256": "…" }
  ]
}
```

- `blobs[].id` is the **resource id** (independent of filename).
- `blobs[].path` is informational/provenance (rebuilt on unpack).
- `offset`/`length` are relative to the **uncompressed** data region.
- `sha256` is over the **uncompressed** resource bytes.

## Compression

- The manifest and the data region are each zlib-deflated **only when that is
  smaller** than storing them raw; the corresponding flag bit records this.
- Offsets/lengths and digests are always relative to uncompressed data, so
  compression is transparent to readers.

## Resource indexing

A resource is looked up by id → `(offset, length)` into the uncompressed data
region. Resource ids are stable and filename-independent.

## Limits & guardrails

| Limit | Value |
|---|---|
| Max manifest (stored) | 64 MiB |
| Max blobs | 4096 |
| Max single blob | 512 MiB |
| Max total resource bytes | 2 GiB |
| Decompressed manifest/data | capped at the above; a decompression bomb is rejected |

## Malformed-input behavior

- Lengths are `checked_add`-ed; every blob range is bounds-checked.
- A truncated/inconsistent container is rejected (`package is truncated …`).
- Integrity mismatch is rejected (`resource '…' failed integrity validation`).
- Decompression is size-limited.
- The C ABI catches panics and reports them as errors (`rva_last_error`); the
  parser never aborts the process on malformed input.
- Unknown optional manifest fields are ignored (forward compatibility).

## Compatibility

- A `major` mismatch is rejected; a newer `minor` is accepted.
- Assets declare `formatVersion` (scene schema) and `containerVersion`.

## Not frozen

Package container technology, final signature, media type registration and
optional binary/binary-compact manifest encodings remain open decisions.
