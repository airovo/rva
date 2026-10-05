import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart' show rootBundle;
import 'package:rva_flutter/rva_flutter.dart';

void main() => runApp(const RvaExampleApp());

class RvaExampleApp extends StatelessWidget {
  const RvaExampleApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      title: 'RVA — one asset, any shape',
      home: HomePage(),
    );
  }
}

class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('RVA — one asset, any shape')),
      backgroundColor: const Color(0xFFF8FAFC),
      body: Center(
        child: FutureBuilder<Uint8List>(
          future: rootBundle
              .load('assets/hero.rva')
              .then((data) => data.buffer.asUint8List()),
          builder: (context, snapshot) {
            if (!snapshot.hasData) {
              return const CircularProgressIndicator();
            }
            // The box changes shape with the window; the asset recomposes.
            final width = MediaQuery.sizeOf(context).width;
            final aspect = width < 480
                ? 9 / 16
                : width < 820
                    ? 1.0
                    : width < 1400
                        ? 16 / 9
                        : 1920 / 500;
            return ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 1100),
              child: AspectRatio(
                aspectRatio: aspect,
                child: RVAImageView(bytes: snapshot.data!),
              ),
            );
          },
        ),
      ),
    );
  }
}
