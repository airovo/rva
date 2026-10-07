package dev.rva

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.RadialGradient
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.Typeface
import com.caverock.androidsvg.SVG
import java.io.ByteArrayInputStream
import kotlin.math.cos
import kotlin.math.max
import kotlin.math.sin
import org.json.JSONObject

// Rasterizes a resolved scene with Android Canvas.
//
// The core decides WHAT to draw (RVAImage.resolveJSON); this renderer decides
// HOW, using Android's native graphics stack. Coordinates share the scene's
// top-left origin, so no flip is needed (unlike Core Graphics).

object RVARenderer {
    fun draw(canvas: Canvas, image: RVAImage, width: Float, height: Float) {
        val scene = JSONObject(image.resolveJSON(width.toInt(), height.toInt()))

        canvas.drawColor(Color.WHITE)

        val paint = Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG)
        val background = scene.optJSONObject("background")
        if (background != null) {
            when (background.optString("type")) {
                "image" -> {
                    val bitmap = decode(image, background.getString("resource"))
                    if (bitmap != null) drawCover(canvas, bitmap, width, height)
                }
                "paint" -> drawPaintFill(canvas, paint, background.optJSONObject("paint"), 0f, 0f, width, height)
            }
        }

        val items = scene.optJSONArray("items") ?: return
        for (index in 0 until items.length()) {
            val item = items.getJSONObject(index)
            paint.alpha = ((item.optDouble("opacity", 1.0) * 255).toInt()).coerceIn(0, 255)
            when (item.getString("type")) {
                "text" -> drawText(canvas, item, paint)
                "vector" -> drawVector(canvas, image, item)
                "paint" -> drawPaintFill(
                    canvas, paint, item.optJSONObject("paint"),
                    item.getDouble("x").toFloat(), item.getDouble("y").toFloat(),
                    item.getDouble("w").toFloat(), item.getDouble("h").toFloat()
                )
                else -> drawRaster(canvas, image, item, paint)
            }
        }
    }

    /** Fill a box with a solid colour or gradient. */
    private fun drawPaintFill(canvas: Canvas, paint: Paint, fill: JSONObject?, x: Float, y: Float, w: Float, h: Float) {
        val shader = shaderFor(fill, x, y, w, h)
        paint.shader = shader
        if (shader == null) {
            paint.color =
                if (fill != null && fill.optString("type") == "color") Color.parseColor(fill.getString("color"))
                else Color.WHITE
        }
        canvas.drawRect(x, y, x + w, y + h, paint)
        paint.shader = null
    }

    /** Apply a text fill (gradient wins, then explicit colour, then a default). */
    private fun applyTextFill(
        fill: JSONObject?, color: String, fallback: Int,
        x: Float, y: Float, w: Float, h: Float, paint: Paint
    ) {
        val shader = shaderFor(fill, x, y, w, h)
        if (shader != null) {
            paint.shader = shader
            return
        }
        paint.shader = null
        val solid = when {
            fill != null && fill.optString("type") == "color" -> fill.optString("color")
            else -> color
        }
        paint.color = if (solid.isNotEmpty()) Color.parseColor(solid) else fallback
    }

    /** A gradient shader matching the reference geometry (top-left origin). */
    private fun shaderFor(fill: JSONObject?, x: Float, y: Float, w: Float, h: Float): Shader? {
        if (fill == null) return null
        val type = fill.optString("type")
        if (type != "linearGradient" && type != "radialGradient") return null
        val stops = fill.optJSONArray("stops") ?: return null
        val n = stops.length()
        if (n == 0) return null
        val colors = IntArray(n)
        val positions = FloatArray(n)
        for (i in 0 until n) {
            val stop = stops.getJSONObject(i)
            colors[i] = Color.parseColor(stop.getString("color"))
            positions[i] = stop.optDouble("offset", 0.0).toFloat()
        }
        return if (type == "radialGradient") {
            RadialGradient(x + w / 2f, y + h / 2f, max(w, h) / 2f, colors, positions, Shader.TileMode.CLAMP)
        } else {
            val rad = Math.toRadians(fill.optDouble("angle", 0.0))
            val x1 = x + (0.5f - 0.5f * cos(rad).toFloat()) * w
            val y1 = y + (0.5f - 0.5f * sin(rad).toFloat()) * h
            val x2 = x + (0.5f + 0.5f * cos(rad).toFloat()) * w
            val y2 = y + (0.5f + 0.5f * sin(rad).toFloat()) * h
            LinearGradient(x1, y1, x2, y2, colors, positions, Shader.TileMode.CLAMP)
        }
    }

    private fun drawRaster(canvas: Canvas, image: RVAImage, item: JSONObject, paint: Paint) {
        val bitmap = decode(image, item.getString("resource")) ?: return
        val rect = RectF(
            item.getDouble("x").toFloat(),
            item.getDouble("y").toFloat(),
            (item.getDouble("x") + item.getDouble("w")).toFloat(),
            (item.getDouble("y") + item.getDouble("h")).toFloat()
        )
        canvas.drawBitmap(bitmap, null, rect, paint)
    }

    private fun drawVector(canvas: Canvas, image: RVAImage, item: JSONObject) {
        val bytes = image.resource(item.getString("resource"))
        val svg = SVG.getFromInputStream(ByteArrayInputStream(bytes))
        svg.setDocumentWidth(item.getDouble("w").toFloat())
        svg.setDocumentHeight(item.getDouble("h").toFloat())
        canvas.save()
        canvas.translate(item.getDouble("x").toFloat(), item.getDouble("y").toFloat())
        val width = svg.documentWidth
        val height = svg.documentHeight
        canvas.clipRect(0f, 0f, width, height)
        svg.renderToCanvas(canvas)
        canvas.restore()
    }

    private fun drawText(canvas: Canvas, item: JSONObject, paint: Paint) {
        val size = item.optDouble("size", 16.0).toFloat()
        val weight = item.optInt("weight", 400)
        paint.textSize = size
        paint.typeface = typefaceFor(weight)
        paint.color = if (item.optString("role") == "subheadline") {
            Color.rgb(51, 65, 85)
        } else {
            Color.rgb(15, 23, 42)
        }
        val lines = mutableListOf<String>()
        val array = item.optJSONArray("lines")
        if (array != null) {
            for (i in 0 until array.length()) lines.add(array.getString(i))
        }
        if (lines.isEmpty()) lines.add(item.optString("value"))

        val ascent = item.optDouble("ascent", size * 0.8).toFloat()
        val lineHeight = item.optDouble("lineHeight", size * 1.08).toFloat()
        val x = item.getDouble("x").toFloat()
        val y = item.getDouble("y").toFloat()
        lines.forEachIndexed { i, line ->
            canvas.drawText(line, x, y + ascent + i * lineHeight, paint)
        }
    }

    private fun drawCover(canvas: Canvas, bitmap: Bitmap, width: Float, height: Float) {
        val scale = maxOf(width / bitmap.width, height / bitmap.height)
        val dw = bitmap.width * scale
        val dh = bitmap.height * scale
        val left = (width - dw) / 2f
        val top = (height - dh) / 2f
        canvas.drawBitmap(
            bitmap,
            null,
            RectF(left, top, left + dw, top + dh),
            Paint(Paint.FILTER_BITMAP_FLAG)
        )
    }

    private fun decode(image: RVAImage, resource: String): Bitmap? {
        val bytes = image.resource(resource)
        return BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
    }

    private fun typefaceFor(weight: Int): Typeface =
        when {
            weight >= 700 -> Typeface.create(Typeface.DEFAULT, Typeface.BOLD)
            weight >= 500 -> Typeface.create(Typeface.DEFAULT, Typeface.BOLD)
            else -> Typeface.DEFAULT
        }
}
