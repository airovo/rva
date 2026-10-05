package dev.rva

// RVA Android/Kotlin adapter.
//
// Binds the shared Rust core through a JNI bridge (ffi/src/jni_bridge.rs) in
// `librva_ffi.so`. Using JNI directly keeps the app free of third-party native
// libraries and lets the shared object be 16 KB page aligned for Android 15+.
//
// Uniform adapter surface (see adapters/contract.json):
//   open(bytes)                -> RVAImage
//   describe()                 -> String
//   resolveJSON(width, height) -> ResolvedScene JSON
//   resource(reference)        -> ByteArray
//
// The core decides WHAT to draw; RVARenderer draws it with Android Canvas.

object RvaNative {
    init {
        System.loadLibrary("rva_ffi")
    }

    external fun nativeOpen(bytes: ByteArray): Long
    external fun nativeFree(handle: Long)
    external fun nativeDescribe(handle: Long): String
    external fun nativeResolve(handle: Long, width: Int, height: Int): String
    external fun nativeRenderPng(handle: Long, width: Int, height: Int): ByteArray
    external fun nativeResource(handle: Long, name: String): ByteArray
    external fun nativeHasResource(handle: Long, name: String): Boolean
    external fun nativeRelativeFor(handle: Long, name: String): String
}

class RVAImage private constructor(private var handle: Long) {

    companion object {
        /** Parse and integrity-validate a .rva package. */
        @JvmStatic
        fun open(bytes: ByteArray): RVAImage {
            val handle = RvaNative.nativeOpen(bytes)
            check(handle != 0L) { "rva_open failed" }
            return RVAImage(handle)
        }
    }

    /** Human-readable asset summary. */
    fun describe(): String = RvaNative.nativeDescribe(handle)

    /** Resolve the asset for a logical viewport, returning ResolvedScene JSON. */
    fun resolveJSON(width: Int, height: Int): String =
        RvaNative.nativeResolve(handle, width, height)

    /** Resolve and rasterize the asset to PNG bytes (rendered by the Rust core). */
    fun renderPng(width: Int, height: Int): ByteArray =
        RvaNative.nativeRenderPng(handle, width, height)

    /** Raw bytes for a declared resource id or raw path. */
    fun resource(reference: String): ByteArray = RvaNative.nativeResource(handle, reference)

    /** Whether a resource reference exists in the package. */
    fun hasResource(reference: String): Boolean = RvaNative.nativeHasResource(handle, reference)

    /** The path a reference resolves to, for MIME detection. */
    fun relativeFor(reference: String): String = RvaNative.nativeRelativeFor(handle, reference)

    /** Release the native handle. */
    fun close() {
        if (handle != 0L) {
            RvaNative.nativeFree(handle)
            handle = 0L
        }
    }
}
