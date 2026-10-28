import java.util.Properties

plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

// Release signing. In CI the upload key comes from environment variables
// (the play-release environment's secrets), so the password never touches
// disk. On a developer machine it comes from android/key.properties, which
// is never committed. Without either, release builds use the debug key,
// which Google Play rejects; with TOTALDROP_REQUIRE_RELEASE_KEY set, that is
// an error instead.
val keyProperties = Properties().apply {
    val f = rootProject.file("key.properties")
    if (f.exists()) f.inputStream().use { load(it) }
}
val envStoreFile: String? = System.getenv("ANDROID_UPLOAD_STORE_FILE")
val signingFromEnv = !envStoreFile.isNullOrEmpty()
val hasReleaseKey = signingFromEnv || keyProperties.getProperty("storeFile") != null
if (!hasReleaseKey && !System.getenv("TOTALDROP_REQUIRE_RELEASE_KEY").isNullOrEmpty()) {
    throw GradleException("No upload key: set ANDROID_UPLOAD_STORE_FILE or android/key.properties")
}

android {
    namespace = "app.totaldrop.totaldrop"
    compileSdk = 37
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        applicationId = "app.totaldrop"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        // Android 15 and newer: only versions that still get security updates.
        minSdk = 35
        targetSdk = 37
        // Uses the version code from pubspec.yaml. When using split APKs, 1000 * ABI_VERSION
        // is added automatically by Flutter. (https://developer.android.com/studio/build/configure-apk-splits#configure-APK-versions)
        // You can force using the value of versionCode by specifying the `-P force-version-code-ignoring-abi=true`
        // flag during build.
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        if (hasReleaseKey) {
            create("release") {
                if (signingFromEnv) {
                    val password = System.getenv("ANDROID_UPLOAD_PASSWORD")
                    storeFile = file(envStoreFile!!)
                    storePassword = password
                    keyAlias = System.getenv("ANDROID_UPLOAD_ALIAS") ?: "upload"
                    keyPassword = password
                } else {
                    storeFile = file(keyProperties.getProperty("storeFile"))
                    storePassword = keyProperties.getProperty("storePassword")
                    keyAlias = keyProperties.getProperty("keyAlias")
                    keyPassword = keyProperties.getProperty("keyPassword")
                }
            }
        }
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.getByName(if (hasReleaseKey) "release" else "debug")
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}
