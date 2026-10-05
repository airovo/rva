import Foundation
import React
import RVAFFI

// RVA React Native native module (iOS).
//
// Thin marshalling over the shared Rust core (RVAFFI.xcframework). It loads the
// `.rva` bytes, opens a core handle, and returns either the resolved scene JSON
// or a base64 PNG rendered by the core.
//
//   JS  ->  RNRva.resolve/renderPng  ->  rva_open/rva_resolve/rva_render_png
@objc(RNRva)
final class RNRva: NSObject {

    @objc
    static func requiresMainQueueSetup() -> Bool { false }

    private func loadAsset(_ uri: String) throws -> Data {
        guard let url = URL(string: uri) else {
            throw NSError(domain: "rva", code: 1, userInfo: [NSLocalizedDescriptionKey: "bad uri \(uri)"])
        }
        switch url.scheme {
        case "http", "https", "file":
            return try Data(contentsOf: url)
        default:
            return try Data(contentsOf: URL(fileURLWithPath: uri))
        }
    }

    private func open(_ data: Data) throws -> OpaquePointer {
        let handle = data.withUnsafeBytes { buffer -> OpaquePointer? in
            guard let base = buffer.bindMemory(to: UInt8.self).baseAddress else { return nil }
            return rva_open(base, buffer.count)
        }
        guard let handle = handle else {
            throw NSError(domain: "rva", code: 2,
                          userInfo: [NSLocalizedDescriptionKey: String(cString: rva_last_error())])
        }
        return handle
    }

    @objc(resolve:width:height:resolver:rejecter:)
    func resolve(_ uri: String,
                 width: NSNumber,
                 height: NSNumber,
                 resolver resolve: @escaping RCTPromiseResolveBlock,
                 rejecter reject: @escaping RCTPromiseRejectBlock) {
        do {
            let handle = try open(loadAsset(uri))
            defer { rva_free_handle(handle) }
            guard let pointer = rva_resolve(handle, width.uint32Value, height.uint32Value) else {
                throw NSError(domain: "rva", code: 3,
                              userInfo: [NSLocalizedDescriptionKey: String(cString: rva_last_error())])
            }
            let json = String(cString: pointer)
            rva_free_string(pointer)
            resolve(json)
        } catch {
            reject("rva_error", error.localizedDescription, error)
        }
    }

    @objc(renderPng:width:height:resolver:rejecter:)
    func renderPng(_ uri: String,
                   width: NSNumber,
                   height: NSNumber,
                   resolver resolve: @escaping RCTPromiseResolveBlock,
                   rejecter reject: @escaping RCTPromiseRejectBlock) {
        do {
            let handle = try open(loadAsset(uri))
            defer { rva_free_handle(handle) }
            var length = 0
            guard let pointer = rva_render_png(handle, width.uint32Value, height.uint32Value, &length),
                  length > 0 else {
                throw NSError(domain: "rva", code: 4,
                              userInfo: [NSLocalizedDescriptionKey: String(cString: rva_last_error())])
            }
            let data = Data(bytes: pointer, count: length)
            rva_free_buffer(pointer, length)
            resolve(data.base64EncodedString())
        } catch {
            reject("rva_error", error.localizedDescription, error)
        }
    }
}
