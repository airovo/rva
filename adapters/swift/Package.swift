// swift-tools-version:5.9
//
// RVA Swift adapter.
//
// Links the shared Rust core as an XCFramework and wraps it in an idiomatic
// Swift API. Rendering uses Core Graphics / Core Text.
//
//   swift build
//   swift run rva-render <hero.rva> <out-dir>

import PackageDescription

// Development builds from the locally built xcframework (`path:`); run
//   scripts/build-native-libs.sh --skip-android
// to produce it. The release workflow rewrites the line below to a
// `url:` + `checksum:` binary target pointing at the GitHub Release asset
// before creating the version tag (see .github/workflows/release-swift.yml).
//
// Do not edit the `rvaFFI` line by hand; keep it a single line.
let rvaFFI = Target.binaryTarget(name: "RVAFFI", path: "RVAFFI.xcframework")

let package = Package(
    name: "RVA",
    platforms: [.macOS(.v13), .iOS(.v15)],
    products: [
        .library(name: "RVA", targets: ["RVA"]),
        .executable(name: "rva-render", targets: ["RVARenderCLI"]),
    ],
    targets: [
        rvaFFI,
        .target(name: "RVA", dependencies: ["RVAFFI"]),
        .executableTarget(name: "RVARenderCLI", dependencies: ["RVA"], path: "Examples/RVARenderCLI"),
    ]
)
