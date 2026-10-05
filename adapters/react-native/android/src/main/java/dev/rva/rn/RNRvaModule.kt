package dev.rva.rn

import android.util.Base64
import com.facebook.react.bridge.Promise
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.bridge.ReactContextBaseJavaModule
import com.facebook.react.bridge.ReactMethod
import dev.rva.RVAImage
import java.io.File
import java.net.URL

// RVA React Native native module (Android).
//
// Thin marshalling over the shared Rust core via the @rva/kotlin JNI bridge.
//   JS -> RNRva.resolve/renderPng -> RVAImage (JNI) -> librva_ffi.so
class RNRvaModule(reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {

    override fun getName() = "RNRva"

    private fun loadBytes(uri: String): ByteArray {
        val url = URL(uri)
        return when (url.protocol) {
            "file" -> File(url.toURI()).readBytes()
            else -> url.openStream().use { it.readBytes() }
        }
    }

    @ReactMethod
    fun resolve(uri: String, width: Int, height: Int, promise: Promise) {
        try {
            val image = RVAImage.open(loadBytes(uri))
            try {
                promise.resolve(image.resolveJSON(width, height))
            } finally {
                image.close()
            }
        } catch (error: Throwable) {
            promise.reject("rva_error", error)
        }
    }

    @ReactMethod
    fun renderPng(uri: String, width: Int, height: Int, promise: Promise) {
        try {
            val image = RVAImage.open(loadBytes(uri))
            try {
                val png = image.renderPng(width, height)
                promise.resolve(Base64.encodeToString(png, Base64.NO_WRAP))
            } finally {
                image.close()
            }
        } catch (error: Throwable) {
            promise.reject("rva_error", error)
        }
    }
}
