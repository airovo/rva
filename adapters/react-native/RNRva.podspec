# React Native iOS native module for RVA.
#
# Links the Rust core (RVAFFI.xcframework) and the Swift/ObjC bridge. Copy the
# xcframework into adapters/react-native/ios first:
#
#   ./adapters/swift/build-xcframework.sh
#   cp -R adapters/swift/RVAFFI.xcframework adapters/react-native/ios/
Pod::Spec.new do |s|
  s.name         = "RNRva"
  s.version      = "0.1.0"
  s.summary      = "RVA native module wrapping the shared Rust core"
  s.license      = "MIT"
  s.authors      = { "RVA" => "dev@rva.dev" }
  s.homepage     = "https://example.com/rva"
  s.platforms    = { :ios => "15.0" }
  s.source       = { :path => "." }
  s.source_files = "ios/**/*.{swift,m,h}"
  s.swift_version = "5.9"
  s.dependency "React-Core"
  s.vendored_frameworks = "ios/RVAFFI.xcframework"
end
