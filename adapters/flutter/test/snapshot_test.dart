import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rva_flutter/rva_flutter.dart';

// Renders the widget to a PNG when RVA_SNAPSHOT is set (visual verification):
//
//   RVA_FFI_LIB=<dylib> RVA_SNAPSHOT=out.png flutter test test/snapshot_test.dart
void main() {
  final outPath = Platform.environment['RVA_SNAPSHOT'];

  testWidgets('writes a snapshot', (tester) async {
    if (outPath == null) return;
    final bytes = File('../../examples/node/hero.rva').readAsBytesSync();
    final key = GlobalKey();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: RepaintBoundary(
              key: key,
              child: SizedBox(
                width: 1080,
                height: 1080,
                child: RVAImageView(bytes: bytes),
              ),
            ),
          ),
        ),
      ),
    );

    // Allow async image/SVG decoding, then pump so painted frames settle.
    await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 600)));
    await tester.pump();
    await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 300)));
    await tester.pump();

    await tester.runAsync(() async {
      final boundary =
          key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      final image = await boundary.toImage(pixelRatio: 1);
      final data = await image.toByteData(format: ui.ImageByteFormat.png);
      File(outPath).writeAsBytesSync(data!.buffer.asUint8List());
    });
  });
}
