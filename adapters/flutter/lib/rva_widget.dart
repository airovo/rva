// RVA Flutter rendering layer.
//
// `RVAImageView` resolves the asset for its own size (via the FFI-backed
// RVAImage) and composes the resolved scene with Flutter widgets: a cover-fit
// background, raster/vector elements, and semantic text.

import 'dart:convert';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'rva_image.dart';

/// Parse an RVA CSS hex colour (`#RGB`, `#RRGGBB`, `#RRGGBBAA`) to a [Color].
Color _parseColor(String hex) {
  var h = hex.replaceFirst('#', '').trim();
  if (h.length == 3) {
    h = h.split('').map((c) => '$c$c').join();
  }
  if (h.length == 6) return Color(int.parse('FF$h', radix: 16));
  if (h.length == 8) {
    // RVA is #RRGGBBAA; Flutter wants 0xAARRGGBB.
    final alpha = h.substring(6, 8);
    return Color(int.parse('$alpha${h.substring(0, 6)}', radix: 16));
  }
  return const Color(0xFF000000);
}

/// Build a gradient shader from a resolved `Paint`, or null for a solid colour.
Shader? _shaderFor(Map<String, dynamic> fill, double w, double h) {
  final type = fill['type'] as String?;
  if (type != 'linearGradient' && type != 'radialGradient') return null;
  final rawStops = (fill['stops'] as List?) ?? const [];
  if (rawStops.isEmpty) return null;
  final colors = <Color>[];
  final stops = <double>[];
  for (final raw in rawStops) {
    final stop = raw as Map;
    colors.add(_parseColor(stop['color'] as String));
    stops.add((stop['offset'] as num).toDouble());
  }
  final rect = Rect.fromLTWH(0, 0, w, h);
  if (type == 'radialGradient') {
    return RadialGradient(colors: colors, stops: stops).createShader(rect);
  }
  final rad = ((fill['angle'] as num?)?.toDouble() ?? 0) * math.pi / 180;
  return LinearGradient(
    begin: Alignment(-math.cos(rad), -math.sin(rad)),
    end: Alignment(math.cos(rad), math.sin(rad)),
    colors: colors,
    stops: stops,
  ).createShader(rect);
}

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
    final lines = (item['lines'] as List?)?.cast<String>() ??
        [item['value'] as String? ?? ''];
    final lineHeight = (item['lineHeight'] as num?)?.toDouble() ?? size * 1.08;
    final align = (item['align'] as String?) ?? 'left';
    final textAlign = align == 'center'
        ? TextAlign.center
        : align == 'right'
            ? TextAlign.right
            : TextAlign.left;

    final w = (item['w'] as num?)?.toDouble() ?? 0;
    final h = (item['h'] as num?)?.toDouble() ?? 0;
    final defaultColor = item['role'] == 'subheadline'
        ? const Color(0xFF334155)
        : const Color(0xFF0F172A);

    // Fill: a gradient (shader) wins over `color`; explicit `color` wins over
    // the role default. Gradient coordinates are relative to the text box.
    final fill = item['fill'] as Map<String, dynamic>?;
    final shader = fill == null ? null : _shaderFor(fill, w, h);
    final colorStr = item['color'] as String?;

    var style = TextStyle(
      fontSize: size,
      height: lineHeight / size,
      fontWeight: _weightFor(weight),
      letterSpacing: (item['letterSpacing'] as num?)?.toDouble(),
      wordSpacing: (item['wordSpacing'] as num?)?.toDouble(),
    );
    if (shader != null) {
      style = style.copyWith(foreground: Paint()..shader = shader);
    } else {
      final solid =
          (fill != null && fill['type'] == 'color') ? fill['color'] as String? : colorStr;
      style = style.copyWith(color: solid != null ? _parseColor(solid) : defaultColor);
    }

    Widget child = Text(lines.join('\n'), textAlign: textAlign, style: style);

    final background = item['background'] as String?;
    if (background != null && background.isNotEmpty) {
      child = Container(color: _parseColor(background), child: child);
    }
    return child;
  }

  FontWeight _weightFor(int weight) {
    final index = (weight ~/ 100).clamp(1, 9) - 1;
    return FontWeight.values[index];
  }
}
