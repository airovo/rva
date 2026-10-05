// RVA Flutter rendering layer.
//
// `RVAImageView` resolves the asset for its own size (via the FFI-backed
// RVAImage) and composes the resolved scene with Flutter widgets: a cover-fit
// background, raster/vector elements, and semantic text.

import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'rva_image.dart';

class RVAImageView extends StatefulWidget {
  const RVAImageView({super.key, required this.bytes});

  final Uint8List bytes;

  @override
  State<RVAImageView> createState() => _RVAImageViewState();
}

class _RVAImageViewState extends State<RVAImageView> {
  late final RVAImage _image;
  Map<String, dynamic>? _scene;
  Size _size = Size.zero;
  final Map<String, Uint8List> _resources = {};

  @override
  void initState() {
    super.initState();
    _image = RVAImage.open(widget.bytes);
  }

  @override
  void dispose() {
    _image.close();
    super.dispose();
  }

  Uint8List _resource(String name) =>
      _resources.putIfAbsent(name, () => _image.resource(name));

  Map<String, dynamic> _resolve(Size size) {
    final json = jsonDecode(
      _image.resolveJSON(size.width.round(), size.height.round()),
    ) as Map<String, dynamic>;
    return json;
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(constraints.maxWidth, constraints.maxHeight);
        if (size.width <= 1 || size.height <= 1) {
          return const SizedBox.shrink();
        }
        if (_scene == null || size != _size) {
          _size = size;
          _scene = _resolve(size);
        }
        return _buildScene(size, _scene!);
      },
    );
  }

  Widget _buildScene(Size size, Map<String, dynamic> scene) {
    final children = <Widget>[];

    final background = scene['background'] as Map<String, dynamic>?;
    if (background != null && background['type'] == 'image') {
      children.add(
        Positioned.fill(
          child: Image.memory(
            _resource(background['resource'] as String),
            fit: BoxFit.cover,
            filterQuality: FilterQuality.medium,
            gaplessPlayback: true,
          ),
        ),
      );
    }

    for (final raw in (scene['items'] as List)) {
      final item = raw as Map<String, dynamic>;
      final opacity = (item['opacity'] as num?)?.toDouble() ?? 1.0;
      Widget child = switch (item['type'] as String) {
        'text' => _text(item),
        'vector' => SvgPicture.memory(
            _resource(item['resource'] as String),
            fit: BoxFit.fill,
          ),
        _ => Image.memory(
            _resource(item['resource'] as String),
            fit: BoxFit.fill,
            filterQuality: FilterQuality.medium,
            gaplessPlayback: true,
          ),
      };
      if (opacity < 1) child = Opacity(opacity: opacity, child: child);

      children.add(
        Positioned(
          left: (item['x'] as num).toDouble(),
          top: (item['y'] as num).toDouble(),
          width: (item['w'] as num).toDouble(),
          height: item['type'] == 'text' ? null : (item['h'] as num).toDouble(),
          child: child,
        ),
      );
    }

    return SizedBox(
      width: size.width,
      height: size.height,
      child: Stack(clipBehavior: Clip.hardEdge, children: children),
    );
  }

  Widget _text(Map<String, dynamic> item) {
    final size = (item['size'] as num?)?.toDouble() ?? 16;
    final weight = (item['weight'] as num?)?.toInt() ?? 400;
    final color = item['role'] == 'subheadline'
        ? const Color(0xFF334155)
        : const Color(0xFF0F172A);
    final lines = (item['lines'] as List?)?.cast<String>() ??
        [item['value'] as String? ?? ''];
    final lineHeight = (item['lineHeight'] as num?)?.toDouble() ?? size * 1.08;

    return Text(
      lines.join('\n'),
      style: TextStyle(
        color: color,
        fontSize: size,
        height: lineHeight / size,
        fontWeight: _weightFor(weight),
      ),
    );
  }

  FontWeight _weightFor(int weight) {
    final index = (weight ~/ 100).clamp(1, 9) - 1;
    return FontWeight.values[index];
  }
}
