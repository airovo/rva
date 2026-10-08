// RVA Android/Kotlin adapter — Android library.
//
// Bundles the prebuilt `librva_ffi.so` (arm64-v8a, x86_64) and binds the C ABI
// through JNI. Assemble the AAR with the Android Gradle plugin:
//
//   ./gradlew :assembleRelease        (from adapters/kotlin)

import com.vanniktech.maven.publish.SonatypeHost

plugins {
    id("com.android.library") version "8.7.3"
    kotlin("android") version "1.9.24"
    id("com.vanniktech.maven.publish") version "0.30.0"
}

// Release version — kept in parity with every other manifest (scripts/check-versions.mjs).
version = "0.1.1"

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

// Publish to Maven Central (Sonatype Central Portal). Credentials + GPG signing
// come from the environment in CI (see .github/workflows/release-kotlin.yml).
mavenPublishing {
    publishToMavenCentral(SonatypeHost.CENTRAL_PORTAL)
    signAllPublications()

    coordinates("tech.airovo", "rva-android", version.toString())

    pom {
        name.set("RVA for Android")
        description.set(
            "RVA (Responsive Visual Asset) Android adapter — JNI bindings to the " +
                "shared Rust core, with an Android Canvas renderer.",
        )
        inceptionYear.set("2026")
        url.set("https://rva.airovo.tech")
        licenses {
            license {
                name.set("MIT")
                url.set("https://opensource.org/licenses/MIT")
                distribution.set("https://opensource.org/licenses/MIT")
            }
        }
        developers {
            developer {
                id.set("airovo")
                name.set("Airovo Technologies")
                url.set("https://airovo.tech")
            }
        }
        scm {
            url.set("https://github.com/airovo/rva")
            connection.set("scm:git:https://github.com/airovo/rva.git")
            developerConnection.set("scm:git:ssh://git@github.com/airovo/rva.git")
        }
    }
}
