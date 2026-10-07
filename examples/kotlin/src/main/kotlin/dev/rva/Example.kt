package dev.rva

// Example: render a .rva asset with the @rva/kotlin adapter.
//
// SCAFFOLD — mirrors the working web/Node examples. Once the adapter is
// implemented the usage is: one asset, any container size.

fun main(args: Array<String>) {
    // A path, a file:// URL or an http(s):// URL.
    val source = args.firstOrNull() ?: "examples/kotlin/hero.rva"
    val image = RVAImage.openSource(source)
    println(image.describe())

    for ((w, h) in listOf(1920 to 500, 1280 to 720, 1080 to 1080, 430 to 932)) {
        val sceneJson = image.resolveJSON(w, h)
        println("$w x $h -> ${sceneJson.take(40)}...")
    }
}
