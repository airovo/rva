import SwiftUI
import RVA

// Example: render a .rva asset with the @rva/swift adapter.
//
// SCAFFOLD — mirrors the working web/Node examples. Once the adapter is
// implemented this is the complete usage: one asset, any container size.

struct HeroView: View {
    var body: some View {
        GeometryReader { proxy in
            RVAResponsiveImage(
                resource: "hero",
                size: proxy.size
            )
        }
    }
}

/// Minimal SwiftUI view that asks the core to resolve the asset for its own
/// size and draws the returned scene.
struct RVAResponsiveImage: View {
    let resource: String
    let size: CGSize

    var body: some View {
        Canvas { context, canvasSize in
            // TODO: image.resolveJSON(width:height:) -> decode scene -> draw
        }
        .background(.white)
    }
}
