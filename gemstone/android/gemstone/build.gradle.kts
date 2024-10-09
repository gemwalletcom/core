plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    id("maven-publish")
    id("com.github.willir.rust.cargo-ndk-android")
}

android {
    namespace = "com.gemwallet.gemstone"
    compileSdk = 34

    defaultConfig {
        minSdk = 28

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
}


cargoNdk {
    targets = arrayListOf("x86_64", "armeabi-v7a", "arm64-v8a")
    module = "../"
    targetDirectory = "../target"
    librariesNames = arrayListOf("libgemstone.so")
    extraCargoBuildArguments = arrayListOf("--lib")
}

dependencies {
    api("net.java.dev.jna:jna:5.15.0@aar"){ artifact { type = "aar" } }
    publishing {
        repositories {
            maven {
                name = "GitHubPackages"
                url = uri("https://maven.pkg.github.com/OWNER/REPOSITORY")
                credentials {
                    username = project.findProperty("github.username") as String? ?: System.getenv("USERNAME")
                    password = project.findProperty("github.token") as String? ?: System.getenv("TOKEN")
                }
            }
        }
    }
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)

    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
}

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("release") {
                from(components["release"])
                groupId = "com.gemwallet.gemstone"
                artifactId = "gemstone"
                version = "1.0.0"
            }
        }
    }
}

publishing {
    repositories {
        maven {
            name = "GPR"
            url = uri("https://maven.pkg.github.com/gemwalletcom/core")
            credentials {
                username = project.findProperty("github.username") as String? ?: System.getenv("GITHUB_USER")
                password = project.findProperty("github.token") as String? ?: System.getenv("GITHUB_TOKEN")
            }
        }
    }
}
