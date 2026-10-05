import Foundation
import RVAFFI

// RVA Swift adapter.
//
// Uniform adapter surface (see adapters/contract.json):
//   open(data)                 -> RVAImage
//   describe()                 -> String
//   resolve(width:height:)     -> ResolvedScene
//   resource(_ reference:)     -> Data
//
// The core decides WHAT to draw; RVARenderer draws it with Core Graphics.

public enum RVAError: Error, CustomStringConvertible {
    case open(String)
    case resolve(String)
    case resource(String)

    public var description: String {
        switch self {
        case .open(let message): return "rva_open: \(message)"
        case .resolve(let message): return "rva_resolve: \(message)"
        case .resource(let message): return "rva_resource: \(message)"
        }
    }
}

/// One responsive visual asset, backed by the shared RVA core.
public final class RVAImage {
    private let handle: OpaquePointer

    public init(data: Data) throws {
        let opened: OpaquePointer? = data.withUnsafeBytes { buffer in
            guard let base = buffer.bindMemory(to: UInt8.self).baseAddress else { return nil }
            return rva_open(base, buffer.count)
        }
        guard let handle = opened else {
            throw RVAError.open(Self.lastError())
        }
        self.handle = handle
    }

    deinit {
        rva_free_handle(handle)
    }

    /// Human-readable asset summary.
    public func describe() -> String {
        takeString(rva_describe(handle))
    }

    /// Resolve the asset for a logical viewport.
    public func resolve(width: Int, height: Int) throws -> ResolvedScene {
        guard let raw = rva_resolve(handle, UInt32(width), UInt32(height)) else {
            throw RVAError.resolve(Self.lastError())
        }
        let json = takeString(raw)
        guard let data = json.data(using: .utf8) else {
            throw RVAError.resolve("invalid UTF-8 scene")
        }
        return try JSONDecoder().decode(ResolvedScene.self, from: data)
    }

    /// Resolve and rasterize the asset to PNG data (rendered by the Rust core).
    public func renderPNG(width: Int, height: Int) throws -> Data {
        var length = 0
        let pointer = rva_render_png(handle, UInt32(width), UInt32(height), &length)
        guard let pointer = pointer, length > 0 else {
            throw RVAError.resolve(Self.lastError())
        }
        let data = Data(bytes: pointer, count: length)
        rva_free_buffer(pointer, length)
        return data
    }

    /// Raw bytes for a declared resource id or raw path.
    public func resource(_ reference: String) throws -> Data {
        var length = 0
        let pointer = reference.withCString { rva_resource(handle, $0, &length) }
        guard let pointer = pointer, length > 0 else {
            throw RVAError.resource(Self.lastError())
        }
        let data = Data(bytes: pointer, count: length)
        rva_free_buffer(pointer, length)
        return data
    }

    /// Whether a resource reference exists in the package.
    public func hasResource(_ reference: String) -> Bool {
        reference.withCString { rva_has_resource(handle, $0) }
    }

    /// The path a reference resolves to, for MIME detection.
    public func relativeFor(_ reference: String) -> String {
        reference.withCString { takeString(rva_relative_for(handle, $0)) }
    }

    /// The current thread's last error message.
    public static func lastError() -> String {
        guard let pointer = rva_last_error() else { return "unknown error" }
        return String(cString: pointer)
    }

    private func takeString(_ pointer: UnsafeMutablePointer<CChar>?) -> String {
        guard let pointer = pointer else { return "" }
        let string = String(cString: pointer)
        rva_free_string(pointer)
        return string
    }
}
