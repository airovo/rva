// RVA Flutter adapter — dart:ffi binding to the shared Rust core.
//
// Loads `librva_ffi` (the same C ABI used by the Swift and Android adapters).
// The core decides WHAT to draw; a CustomPainter (rva_widget.dart) draws it.
//
// Uniform adapter surface (see adapters/contract.json):
//   open(bytes)                -> RVAImage
//   openSource(source)         -> RVAImage   (path | file:// | http(s)://)
//   describe()                 -> String
//   resolveJSON(width, height) -> ResolvedScene JSON
//   resource(reference)        -> Uint8List

import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

// --- C signatures -----------------------------------------------------------

typedef _OpenC = Pointer<Void> Function(Pointer<Uint8>, IntPtr);
typedef _OpenDart = Pointer<Void> Function(Pointer<Uint8>, int);

typedef _VoidHandleC = Void Function(Pointer<Void>);
typedef _VoidHandleDart = void Function(Pointer<Void>);

typedef _DescribeC = Pointer<Utf8> Function(Pointer<Void>);
typedef _DescribeDart = Pointer<Utf8> Function(Pointer<Void>);

typedef _ResolveC = Pointer<Utf8> Function(Pointer<Void>, Uint32, Uint32);
typedef _ResolveDart = Pointer<Utf8> Function(Pointer<Void>, int, int);

typedef _ResourceC = Pointer<Uint8> Function(Pointer<Void>, Pointer<Utf8>, Pointer<UintPtr>);
typedef _ResourceDart = Pointer<Uint8> Function(Pointer<Void>, Pointer<Utf8>, Pointer<UintPtr>);

typedef _HasC = Uint8 Function(Pointer<Void>, Pointer<Utf8>);
typedef _HasDart = int Function(Pointer<Void>, Pointer<Utf8>);

typedef _RelativeC = Pointer<Utf8> Function(Pointer<Void>, Pointer<Utf8>);
typedef _RelativeDart = Pointer<Utf8> Function(Pointer<Void>, Pointer<Utf8>);

typedef _LastErrorC = Pointer<Utf8> Function();
typedef _LastErrorDart = Pointer<Utf8> Function();

typedef _FreeStringC = Void Function(Pointer<Utf8>);
typedef _FreeStringDart = void Function(Pointer<Utf8>);

typedef _FreeBufferC = Void Function(Pointer<Uint8>, UintPtr);
typedef _FreeBufferDart = void Function(Pointer<Uint8>, int);

DynamicLibrary _loadLibrary() {
  final override = Platform.environment['RVA_FFI_LIB'];
  if (override != null && override.isNotEmpty) {
    return DynamicLibrary.open(override);
  }
  // Apple: the core is bundled as the dynamic RVAFFI.framework (see the ios/ and
  // macos/ podspecs). Android/Linux: librva_ffi.so from the plugin jniLibs.
  if (Platform.isMacOS || Platform.isIOS) {
    return DynamicLibrary.open('RVAFFI.framework/RVAFFI');
  }
  if (Platform.isAndroid || Platform.isLinux) return DynamicLibrary.open('librva_ffi.so');
  if (Platform.isWindows) return DynamicLibrary.open('rva_ffi.dll');
  throw UnsupportedError('RVA: unsupported platform');
}

final DynamicLibrary _lib = _loadLibrary();

final _rvaOpen = _lib.lookupFunction<_OpenC, _OpenDart>('rva_open');
final _rvaFreeHandle = _lib.lookupFunction<_VoidHandleC, _VoidHandleDart>('rva_free_handle');
final _rvaDescribe = _lib.lookupFunction<_DescribeC, _DescribeDart>('rva_describe');
final _rvaResolve = _lib.lookupFunction<_ResolveC, _ResolveDart>('rva_resolve');
final _rvaResource = _lib.lookupFunction<_ResourceC, _ResourceDart>('rva_resource');
final _rvaHasResource = _lib.lookupFunction<_HasC, _HasDart>('rva_has_resource');
final _rvaRelativeFor = _lib.lookupFunction<_RelativeC, _RelativeDart>('rva_relative_for');
final _rvaLastError = _lib.lookupFunction<_LastErrorC, _LastErrorDart>('rva_last_error');
final _rvaFreeString = _lib.lookupFunction<_FreeStringC, _FreeStringDart>('rva_free_string');
final _rvaFreeBuffer = _lib.lookupFunction<_FreeBufferC, _FreeBufferDart>('rva_free_buffer');

String _take(Pointer<Utf8> pointer) {
  if (pointer == nullptr) return '';
  final string = pointer.toDartString();
  _rvaFreeString(pointer);
  return string;
}

/// One responsive visual asset, backed by the shared RVA core.
class RVAImage {
  RVAImage._(this._handle);

  final Pointer<Void> _handle;

  /// Parse and integrity-validate a .rva package.
  factory RVAImage.open(Uint8List bytes) {
    final pointer = malloc<Uint8>(bytes.length);
    try {
      pointer.asTypedList(bytes.length).setAll(0, bytes);
      final handle = _rvaOpen(pointer, bytes.length);
      if (handle == nullptr) {
        throw StateError('rva_open failed: ${RVAImage.lastError()}');
      }
      return RVAImage._(handle);
    } finally {
      malloc.free(pointer);
    }
  }

  /// Open from a filesystem path, a `file://` URL or an `http(s)://` URL.
  /// Convenience over the normative [RVAImage.open] (bytes).
  static Future<RVAImage> openSource(String source) async =>
      RVAImage.open(await readSource(source));

  /// Read `.rva` bytes from a path, `file://` or `http(s)://` URL.
  static Future<Uint8List> readSource(String source) async {
    final uri = Uri.tryParse(source);
    if (uri != null &&
        uri.hasScheme &&
        (uri.scheme == 'file' || uri.scheme == 'http' || uri.scheme == 'https')) {
      if (uri.scheme == 'file') {
        return File.fromUri(uri).readAsBytes();
      }
      final client = HttpClient();
      try {
        final request = await client.getUrl(uri);
        final response = await request.close();
        if (response.statusCode != HttpStatus.ok) {
          throw HttpException('HTTP ${response.statusCode} for $source');
        }
        final builder = BytesBuilder(copy: false);
        await for (final chunk in response) {
          builder.add(chunk);
        }
        return builder.takeBytes();
      } finally {
        client.close(force: true);
      }
    }
    // No recognised scheme: treat as a filesystem path.
    return File(source).readAsBytes();
  }

  /// Human-readable asset summary.
  String describe() => _take(_rvaDescribe(_handle));

  /// Resolve the asset for a logical viewport, returning ResolvedScene JSON.
  String resolveJSON(int width, int height) {
    final pointer = _rvaResolve(_handle, width, height);
    if (pointer == nullptr) {
      throw StateError('rva_resolve failed: ${RVAImage.lastError()}');
    }
    return _take(pointer);
  }

  /// Raw bytes for a declared resource id or raw path.
  Uint8List resource(String reference) {
    final name = reference.toNativeUtf8();
    final length = malloc<UintPtr>();
    try {
      final data = _rvaResource(_handle, name, length);
      if (data == nullptr) {
        throw StateError('rva_resource failed: ${RVAImage.lastError()}');
      }
      final bytes = Uint8List.fromList(data.asTypedList(length.value));
      _rvaFreeBuffer(data, length.value);
      return bytes;
    } finally {
      malloc.free(name);
      malloc.free(length);
    }
  }

  /// Whether a resource reference exists in the package.
  bool hasResource(String reference) {
    final name = reference.toNativeUtf8();
    try {
      return _rvaHasResource(_handle, name) != 0;
    } finally {
      malloc.free(name);
    }
  }

  /// The path a reference resolves to, for MIME detection.
  String relativeFor(String reference) {
    final name = reference.toNativeUtf8();
    try {
      return _take(_rvaRelativeFor(_handle, name));
    } finally {
      malloc.free(name);
    }
  }

  /// Release the native handle.
  void close() => _rvaFreeHandle(_handle);

  /// The current thread's last error message.
  static String lastError() {
    final pointer = _rvaLastError();
    if (pointer == nullptr) return 'unknown error';
    return pointer.toDartString();
  }
}
