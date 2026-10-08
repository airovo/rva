#
# RVA Flutter adapter (macOS). Vendors the prebuilt dynamic core framework.
#
Pod::Spec.new do |s|
  s.name             = 'rva_flutter'
  s.version          = '0.1.1'
  s.summary          = 'RVA Flutter adapter (dart:ffi).'
  s.description      = <<-DESC
RVA (Responsive Visual Asset) Flutter adapter — dart:ffi binding to the shared Rust core.
                       DESC
  s.homepage         = 'https://rva.airovo.tech'
  s.license          = { :file => '../LICENSE' }
  s.author           = { 'Airovo Technologies' => 'dev@airovo.tech' }
  s.source           = { :path => '.' }
  s.dependency 'FlutterMacOS'
  s.platform = :osx, '13.0'
  s.vendored_frameworks = '../RVAFFI.xcframework'
  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES' }
  s.swift_version = '5.0'
end
