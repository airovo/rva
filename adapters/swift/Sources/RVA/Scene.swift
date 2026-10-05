import Foundation

// Decoded form of the core's ResolvedScene JSON. Fields are a superset of the
// discriminated union so decoding is tolerant across element types.

public struct FocalRegion: Decodable {
    public let name: String
    public let x: Double
    public let y: Double
    public let w: Double
    public let h: Double
    public let hard: Bool
}

public struct ResolvedItem: Decodable {
    public let id: String
    public let role: String?
    public let z: Int
    public let x: Double
    public let y: Double
    public let w: Double
    public let h: Double
    public let opacity: Double
    public let mask: String?
    public let type: String

    // image / vector
    public let resource: String?
    public let fit: String?

    // text
    public let value: String?
    public let fontFamily: String?
    public let weight: Int?
    public let size: Double?
    public let lineHeight: Double?
    public let ascent: Double?
    public let lines: [String]?
}

public struct ResolvedScene: Decodable {
    public let topology: String
    public let width: Int
    public let height: Int
    public let topologyFallback: Bool
    public let degraded: Bool
    public let hidden: [String]
    public let viability: Double
    public let background: ResolvedItem?
    public let items: [ResolvedItem]
    public let diagnostics: [String]
}
