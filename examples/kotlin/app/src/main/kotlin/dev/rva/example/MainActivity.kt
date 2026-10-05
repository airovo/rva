package dev.rva.example

import android.app.Activity
import android.content.Context
import android.graphics.Canvas
import android.os.Bundle
import android.view.View
import dev.rva.RVAImage
import dev.rva.RVARenderer

// Example: consume one .rva asset with the @rva/kotlin adapter.
//
// One asset, any shape: the core resolves the composition for the view's size
// and the adapter draws it with Android Canvas.

class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val bytes = assets.open("hero.rva").readBytes()
        val image = RVAImage.open(bytes)
        setContentView(HeroView(this, image))
    }
}

private class HeroView(context: Context, private val image: RVAImage) : View(context) {
    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        RVARenderer.draw(canvas, image, width.toFloat(), height.toFloat())
    }
}
