package dev.rva

// Example: render a .rva asset with the @rva/kotlin adapter.
//
// SCAFFOLD — mirrors the working web/Node examples. Once the adapter is
// implemented the usage is: one asset, any container size.

fun main() {
    val bytes = java.io.File("examples/kotlin/hero.rva").readBytes()
    val image = RVAImage.open(bytes)
    println(image.describe())

    for ((w, h) in listOf(1920 to 500, 1280 to 720, 1080 to 1080, 430 to 932)) {
        val sceneJson = image.resolveJSON(w, h)
        println("$w x $h -> ${sceneJson.take(40)}...")
    }
}
