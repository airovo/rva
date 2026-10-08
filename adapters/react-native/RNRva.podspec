# React Native iOS native module for RVA.
#
# Links the shared Rust core, shipped as a *dynamic* RVAFFI.xcframework. Build it
# (and copy it into ios/) with:
#
#   scripts/build-native-libs.sh --skip-android
#
# CocoaPods embeds the dynamic framework into the app target; the runpath below
# keeps @rpath/RVAFFI.framework/RVAFFI resolvable at load time.
Pod::Spec.new do |s|
  s.name         = "RNRva"
  # Kept in parity with every other manifest (scripts/check-versions.mjs).
  s.version      = "0.1.1"
  s.summary      = "RVA React Native native module wrapping the shared Rust core"
  s.description  = "Resolves and rasterizes .rva assets with the shared RVA core " \
                   "over the C ABI (rva_open / rva_resolve / rva_render_png)."
  s.license      = "MIT"
  s.authors      = { "Airovo Technologies" => "dev@airovo.tech" }
  s.homepage     = "https://rva.airovo.tech"
  s.platforms    = { :ios => "15.0" }
  s.source       = { :path => "." }
  # Only the bridge sources — never glob into the .xcframework bundle.
  s.source_files = "ios/RNRva.{swift,m,h}"
  s.swift_version = "5.9"
  s.dependency "React-Core"
  s.vendored_frameworks = "ios/RVAFFI.xcframework"
  # The vendored framework is dynamic; make sure @rpath resolves in the app.
  s.user_target_xcconfig = {
    "LD_RUNPATH_SEARCH_PATHS" => "$(inherited) @executable_path/Frameworks",
  }
end
