// FFI smoke test for the Flutter adapter (no Flutter engine required).
//
//   RVA_FFI_LIB=<path to librva_ffi.dylib> dart run tool/smoke.dart <asset>
//
// <asset> may be a filesystem path, a file:// URL or an http(s):// URL.

import 'package:rva_flutter/rva_image.dart';

Future<void> main(List<String> args) async {
  final source = args.isNotEmpty ? args[0] : 'hero.rva';
  final image = await RVAImage.openSource(source);
  print(image.describe());

  for (final size in [
    [1920, 500],
    [1280, 720],
    [1080, 1080],
    [430, 932],
  ]) {
    final json = image.resolveJSON(size[0], size[1]);
    final topology = RegExp(r'"topology":"([^"]+)"').firstMatch(json)?.group(1);
    final viability = RegExp(r'"viability":([0-9.]+)').firstMatch(json)?.group(1);
    print('${size[0]}x${size[1]} topology=$topology viability=$viability');
  }

  print(
      'bgLandscape: ${image.resource('bgLandscape').length} bytes, '
      'has=${image.hasResource('bgLandscape')}, '
      'relative=${image.relativeFor('bgLandscape')}');
  image.close();
}
