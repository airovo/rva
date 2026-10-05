import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rva_flutter/rva_flutter.dart';

void main() {
  testWidgets('resolves and paints a .rva asset', (tester) async {
    final bytes = File('../../examples/node/hero.rva').readAsBytesSync();

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SizedBox(
            width: 1080,
            height: 1080,
            child: RVAImageView(bytes: bytes),
          ),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 300));

    expect(find.byType(RVAImageView), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
