package dev.rva

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.RectF
import android.graphics.Typeface
import com.caverock.androidsvg.SVG
import java.io.ByteArrayInputStream
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

        val background = scene.optJSONObject("background")
        if (background != null && background.optString("type") == "image") {
            val bitmap = decode(image, background.getString("resource"))
            if (bitmap != null) drawCover(canvas, bitmap, width, height)
        }

        val items = scene.optJSONArray("items") ?: return
        val paint = Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG)
        for (index in 0 until items.length()) {
            val item = items.getJSONObject(index)
            paint.alpha = ((item.optDouble("opacity", 1.0) * 255).toInt()).coerceIn(0, 255)
            when (item.getString("type")) {
                "text" -> drawText(canvas, item, paint)
                "vector" -> drawVector(canvas, image, item)
                else -> drawRaster(canvas, image, item, paint)
            }
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
