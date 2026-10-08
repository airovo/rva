# RVA for Swift

> **Moved.** The Swift package now lives in its own repository so it can version
> independently: **<https://github.com/airovo/rva-swift>**.

There is no Swift source in this repository. This repo builds and publishes the
Apple binary (`RVAFFI.xcframework`) that the Swift package pins; see
[`scripts/build-xcframework.sh`](../../scripts/build-xcframework.sh) and the
[`Release (xcframework)`](../../.github/workflows/release-xcframework.yml) workflow.

## Install

```swift
// Package.swift
dependencies: [
    .package(url: "https://github.com/airovo/rva-swift", from: "0.1.0")
]
```

Full usage and API reference: <https://github.com/airovo/rva-swift>.

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, built by
[Airovo Technologies](https://airovo.tech).
