#if canImport(AppKit)
import AppKit
import CoreGraphics
import CoreText
import Foundation
import ImageIO
import UniformTypeIdentifiers

/// Rasterizes a resolved scene with Core Graphics / Core Text.
///
/// The core decides WHAT to draw; this renderer decides HOW, using the native
/// Apple graphics stack. Coordinates are mapped from the scene's top-left origin
/// to Core Graphics' bottom-left origin.
public enum RVARenderer {
    /// Resolve and rasterize a viewport to PNG data.
    public static func pngData(
        image: RVAImage,
        width: Int,
        height: Int,
        scale: CGFloat = 2
    ) throws -> Data {
        let scene = try image.resolve(width: width, height: height)
        let cgImage = try render(image: image, scene: scene, scale: scale)
        return try encodePNG(cgImage)
    }

    public static func render(
        image: RVAImage,
        scene: ResolvedScene,
        scale: CGFloat = 2
    ) throws -> CGImage {
        let pixelWidth = Int((CGFloat(scene.width) * scale).rounded())
        let pixelHeight = Int((CGFloat(scene.height) * scale).rounded())
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        guard
            let context = CGContext(
                data: nil,
                width: pixelWidth,
                height: pixelHeight,
                bitsPerComponent: 8,
                bytesPerRow: 0,
                space: colorSpace,
                bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            )
        else {
            throw RVAError.resolve("could not create bitmap context")
        }

        context.scaleBy(x: scale, y: scale)
        context.setFillColor(CGColor(red: 1, green: 1, blue: 1, alpha: 1))
        context.fill(CGRect(x: 0, y: 0, width: scene.width, height: scene.height))

        let W = CGFloat(scene.width)
        let H = CGFloat(scene.height)

        if let background = scene.background,
           background.type == "image",
           let resource = background.resource,
           let cgImage = try? decode(image: image, resource: resource) {
            let iw = CGFloat(cgImage.width)
            let ih = CGFloat(cgImage.height)
            let cover = max(W / iw, H / ih)
            let dw = iw * cover
            let dh = ih * cover
            context.saveGState()
            context.clip(to: CGRect(x: 0, y: 0, width: W, height: H))
            context.draw(cgImage, in: CGRect(x: (W - dw) / 2, y: (H - dh) / 2, width: dw, height: dh))
            context.restoreGState()
        }

        for item in scene.items {
            context.setAlpha(CGFloat(item.opacity))
            let rect = CGRect(x: item.x, y: H - item.y - item.h, width: item.w, height: item.h)

            if item.type == "text" {
                drawText(item, in: context, sceneHeight: H)
                continue
            }

            guard
                let resource = item.resource,
                let cgImage = try? decode(image: image, resource: resource)
            else { continue }

            if let maskName = item.mask, let mask = try? decode(image: image, resource: maskName) {
                context.saveGState()
                context.clip(to: rect, mask: mask)
                context.draw(cgImage, in: rect)
                context.restoreGState()
            } else {
                context.draw(cgImage, in: rect)
            }
        }
        context.setAlpha(1)

        guard let result = context.makeImage() else {
            throw RVAError.resolve("could not snapshot context")
        }
        return result
    }

    private static func drawText(_ item: ResolvedItem, in context: CGContext, sceneHeight H: CGFloat) {
        let size = CGFloat(item.size ?? 16)
        let font = systemFont(size: size, weight: item.weight ?? 400)
        let color: NSColor = item.role == "subheadline"
            ? NSColor(calibratedRed: 0.2, green: 0.25, blue: 0.33, alpha: 1)
            : NSColor(calibratedRed: 0.06, green: 0.09, blue: 0.16, alpha: 1)

        let lines = (item.lines?.isEmpty == false ? item.lines! : [item.value ?? ""])
        let ascent = CGFloat(item.ascent ?? Double(size) * 0.8)
        let lineHeight = CGFloat(item.lineHeight ?? Double(size) * 1.08)

        for (index, line) in lines.enumerated() {
            let top = CGFloat(item.y) + ascent + CGFloat(index) * lineHeight
            let baseline = H - top
            let attributed = NSAttributedString(
                string: line,
                attributes: [.font: font, .foregroundColor: color]
            )
            let ctLine = CTLineCreateWithAttributedString(attributed)
            context.textPosition = CGPoint(x: item.x, y: baseline)
            CTLineDraw(ctLine, context)
        }
    }

    private static func systemFont(size: CGFloat, weight: Int) -> NSFont {
        let weight: NSFont.Weight = weight >= 700 ? .bold : (weight >= 500 ? .medium : .regular)
        return NSFont.systemFont(ofSize: size, weight: weight)
    }

    private static func decode(image: RVAImage, resource: String) throws -> CGImage {
        let data = try image.resource(resource)
        if image.relativeFor(resource).lowercased().hasSuffix(".svg") {
            guard let nsImage = NSImage(data: data) else {
                throw RVAError.resource("could not decode SVG \(resource)")
            }
            var rect = NSRect(origin: .zero, size: nsImage.size)
            guard let cgImage = nsImage.cgImage(forProposedRect: &rect, context: nil, hints: nil) else {
                throw RVAError.resource("could not rasterize SVG \(resource)")
            }
            return cgImage
        }
        guard
            let source = CGImageSourceCreateWithData(data as CFData, nil),
            let cgImage = CGImageSourceCreateImageAtIndex(source, 0, nil)
        else {
            throw RVAError.resource("could not decode image \(resource)")
        }
        return cgImage
    }

    private static func encodePNG(_ image: CGImage) throws -> Data {
        let data = NSMutableData()
        guard
            let destination = CGImageDestinationCreateWithData(
                data, UTType.png.identifier as CFString, 1, nil
            )
        else {
            throw RVAError.resolve("could not create PNG destination")
        }
        CGImageDestinationAddImage(destination, image, nil)
        guard CGImageDestinationFinalize(destination) else {
            throw RVAError.resolve("could not finalize PNG")
        }
        return data as Data
    }
}
#endif
