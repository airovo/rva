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

let package = Package(
    name: "RVA",
    platforms: [.macOS(.v13), .iOS(.v15)],
    products: [
        .library(name: "RVA", targets: ["RVA"]),
        .executable(name: "rva-render", targets: ["RVARenderCLI"]),
    ],
    targets: [
        .binaryTarget(name: "RVAFFI", path: "RVAFFI.xcframework"),
        .target(name: "RVA", dependencies: ["RVAFFI"]),
        .executableTarget(name: "RVARenderCLI", dependencies: ["RVA"], path: "Examples/RVARenderCLI"),
    ]
)
