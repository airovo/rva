// RVA Android/Kotlin adapter — Android library.
//
// Bundles the prebuilt `librva_ffi.so` (arm64-v8a, x86_64) and wraps the C ABI
// with JNA. Assemble the AAR with the Android Gradle plugin:
//
//   ./gradlew :assembleRelease        (from adapters/kotlin)

plugins {
    id("com.android.library") version "8.7.3"
    kotlin("android") version "1.9.24"
}

android {
    namespace = "dev.rva"
    compileSdk = 35
    buildToolsVersion = "35.0.0"

    defaultConfig {
        minSdk = 24
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets["main"].jniLibs.srcDirs("src/main/jniLibs")
}

dependencies {
    implementation("com.caverock:androidsvg-aar:1.4")
}
