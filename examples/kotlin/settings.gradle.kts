pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "rva-example-kotlin"

include(":app")
include(":rva")
project(":rva").projectDir = file("../../adapters/kotlin")
