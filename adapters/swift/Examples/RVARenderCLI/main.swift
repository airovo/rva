import Foundation
import RVA

// Example CLI for the Swift adapter: load one .rva and render the canonical
// viewport matrix to PNG using Core Graphics.
//
//   swift run rva-render <asset> [out-dir]
//
// <asset> may be a filesystem path, a file:// URL or an http(s):// URL.

let arguments = CommandLine.arguments
guard arguments.count >= 2 else {
    FileHandle.standardError.write("usage: rva-render <asset.rva> [outDir]\n".data(using: .utf8)!)
    exit(1)
}

let assetPath = arguments[1]
let outDir = arguments.count >= 3 ? arguments[2] : "out"
try? FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)

let image = try RVAImage(source: assetPath)
print(image.describe())

let sizes: [(Int, Int, String)] = [
    (1920, 500, "wide"),
    (1280, 720, "desktop"),
    (768, 1024, "tablet"),
    (1080, 1080, "square"),
    (430, 932, "mobile"),
    (2560, 800, "ultrawide"),
]

for (width, height, name) in sizes {
    let scene = try image.resolve(width: width, height: height)
    let png = try RVARenderer.pngData(image: image, width: width, height: height)
    let file = "\(outDir)/\(name)-\(width)x\(height).png"
    try png.write(to: URL(fileURLWithPath: file))
    print(
        "\(width)x\(height) topology=\(scene.topology) " +
        "viability=\(String(format: "%.2f", scene.viability)) -> \(file)"
    )
}
