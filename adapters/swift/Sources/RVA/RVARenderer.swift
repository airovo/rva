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

        if let background = scene.background {
            if background.type == "image",
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
            } else if background.type == "paint", let paint = background.paint {
                applyPaint(paint, in: context, rect: CGRect(x: 0, y: 0, width: W, height: H))
            }
        }

        for item in scene.items {
            context.setAlpha(CGFloat(item.opacity))
            let rect = CGRect(x: item.x, y: H - item.y - item.h, width: item.w, height: item.h)

            if item.type == "text" {
                drawText(item, in: context, sceneHeight: H)
                continue
            }

            if item.type == "paint", let paint = item.paint {
                applyPaint(paint, in: context, rect: rect)
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
        let lines = (item.lines?.isEmpty == false ? item.lines! : [item.value ?? ""])
        let ascent = CGFloat(item.ascent ?? Double(size) * 0.8)
        let lineHeight = CGFloat(item.lineHeight ?? Double(size) * 1.08)
        let align = item.align ?? "left"
        let boxX = CGFloat(item.x)
        let boxY = CGFloat(item.y)
        let boxW = CGFloat(item.w)
        let boxH = CGFloat(item.h)

        // Optional solid text-box background.
        if let background = item.background, !background.isEmpty, let color = cgColor(background) {
            context.setFillColor(color)
            context.fill(CGRect(x: boxX, y: H - boxY - boxH, width: boxW, height: boxH))
        }

        var attributes: [NSAttributedString.Key: Any] = [.font: font]
        if let spacing = item.letterSpacing, spacing != 0 {
            attributes[.kern] = CGFloat(spacing)
        }

        func line(for text: String, color: NSColor?) -> CTLine {
            var attrs = attributes
            if let color = color { attrs[.foregroundColor] = color }
            return CTLineCreateWithAttributedString(
                NSAttributedString(string: text, attributes: attrs)
            )
        }
        func origin(_ ctLine: CTLine, index: Int) -> CGPoint {
            let width = CGFloat(CTLineGetTypographicBounds(ctLine, nil, nil, nil))
            let x = align == "center" ? boxX + (boxW - width) / 2 : align == "right" ? boxX + boxW - width : boxX
            let top = boxY + ascent + CGFloat(index) * lineHeight
            return CGPoint(x: x, y: H - top)
        }

        // Gradient fill: clip to the glyphs, then paint the gradient over the box.
        if let fill = item.fill, fill.type != "color", let spec = gradientSpec(fill) {
            context.saveGState()
            context.setTextDrawingMode(.clip)
            for (index, text) in lines.enumerated() {
                let ctLine = line(for: text, color: nil)
                context.textPosition = origin(ctLine, index: index)
                CTLineDraw(ctLine, context)
            }
            context.setTextDrawingMode(.fill)
            drawGradient(spec, in: context, rect: CGRect(x: boxX, y: H - boxY - boxH, width: boxW, height: boxH))
            context.restoreGState()
            return
        }

        let solid: NSColor = {
            if let fill = item.fill, fill.type == "color", let hex = fill.color, let cg = cgColor(hex) {
                return NSColor(cgColor: cg) ?? NSColor.black
            }
            if let hex = item.color, let cg = cgColor(hex) {
                return NSColor(cgColor: cg) ?? NSColor.black
            }
            return item.role == "subheadline"
                ? NSColor(calibratedRed: 0.2, green: 0.25, blue: 0.33, alpha: 1)
                : NSColor(calibratedRed: 0.06, green: 0.09, blue: 0.16, alpha: 1)
        }()

        for (index, text) in lines.enumerated() {
            let ctLine = line(for: text, color: solid)
            context.textPosition = origin(ctLine, index: index)
            CTLineDraw(ctLine, context)
        }
    }

    // MARK: - Paint (solid colour / gradient)

    private struct GradientSpec {
        let colors: [CGColor]
        let locations: [CGFloat]
        let radial: Bool
        let angle: Double
    }

    private static func applyPaint(_ paint: Paint, in context: CGContext, rect: CGRect) {
        if paint.type == "color", let hex = paint.color, let color = cgColor(hex) {
            context.setFillColor(color)
            context.fill(rect)
            return
        }
        if let spec = gradientSpec(paint) {
            drawGradient(spec, in: context, rect: rect)
        }
    }

    private static func gradientSpec(_ paint: Paint) -> GradientSpec? {
        guard
            paint.type == "linearGradient" || paint.type == "radialGradient",
            let stops = paint.stops,
            !stops.isEmpty
        else { return nil }
        let colors = stops.map { cgColor($0.color) ?? CGColor(gray: 0, alpha: 1) }
        let locations = stops.map { CGFloat($0.offset) }
        return GradientSpec(
            colors: colors,
            locations: locations,
            radial: paint.type == "radialGradient",
            angle: paint.angle ?? 0
        )
    }

    /// Gradient in the flipped (bottom-left) space, matching the reference
    /// geometry: angle 0 runs left→right.
    private static func drawGradient(_ spec: GradientSpec, in context: CGContext, rect: CGRect) {
        guard
            let gradient = CGGradient(
                colorsSpace: CGColorSpaceCreateDeviceRGB(),
                colors: spec.colors as CFArray,
                locations: spec.locations
            )
        else { return }
        let options: CGGradientDrawingOptions = [.drawsBeforeStartLocation, .drawsAfterEndLocation]
        if spec.radial {
            let center = CGPoint(x: rect.midX, y: rect.midY)
            context.drawRadialGradient(
                gradient,
                startCenter: center, startRadius: 0,
                endCenter: center, endRadius: max(rect.width, rect.height) / 2,
                options: options
            )
        } else {
            let rad = spec.angle * .pi / 180
            let start = CGPoint(
                x: rect.minX + (0.5 - 0.5 * cos(rad)) * rect.width,
                y: rect.maxY - (0.5 - 0.5 * sin(rad)) * rect.height
            )
            let end = CGPoint(
                x: rect.minX + (0.5 + 0.5 * cos(rad)) * rect.width,
                y: rect.maxY - (0.5 + 0.5 * sin(rad)) * rect.height
            )
            context.drawLinearGradient(gradient, start: start, end: end, options: options)
        }
    }

    /// Parse an RVA CSS hex colour (`#RGB`, `#RRGGBB`, `#RRGGBBAA`).
    private static func cgColor(_ hex: String) -> CGColor? {
        var value = hex.trimmingCharacters(in: .whitespaces)
        if value.hasPrefix("#") { value.removeFirst() }
        if value.count == 3 {
            value = value.map { "\($0)\($0)" }.joined()
        }
        guard value.count == 6 || value.count == 8 else { return nil }
        func component(_ offset: Int) -> CGFloat {
            let start = value.index(value.startIndex, offsetBy: offset)
            let end = value.index(start, offsetBy: 2)
            return CGFloat(Int(value[start..<end], radix: 16) ?? 0) / 255
        }
        let alpha = value.count == 8 ? component(6) : 1
        return CGColor(srgbRed: component(0), green: component(2), blue: component(4), alpha: alpha)
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
