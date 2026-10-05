// Android example app for the @rva/kotlin adapter.
//
//   cd examples/kotlin && ./gradlew :app:installDebug

plugins {
    id("com.android.application") version "8.7.3"
    kotlin("android") version "1.9.24"
}

android {
    namespace = "dev.rva.example"
    compileSdk = 35
    buildToolsVersion = "35.0.0"

    defaultConfig {
        applicationId = "dev.rva.example"
        minSdk = 24
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }
}

dependencies {
    implementation(project(":rva"))
}
