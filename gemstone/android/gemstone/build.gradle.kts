import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.library")
    id("maven-publish")
}

val gemstoneRoot = project.projectDir.resolve("../..")
val rustSrcDir = gemstoneRoot.resolve("src")
val cratesDir = gemstoneRoot.resolve("../crates")
val jniLibsDir = project.projectDir.resolve("src/main/jniLibs")
val generatedKotlinDir = project.projectDir.resolve("src/main/java")

android {
    namespace = "com.gemwallet.gemstone"
    compileSdk = 36

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

    publishing {
        singleVariant("release") {
            withSourcesJar()
            withJavadocJar()
        }
        singleVariant("debug") {
            withSourcesJar()
            withJavadocJar()
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main") {
            java.srcDirs(generatedKotlinDir)
            jniLibs.srcDirs(jniLibsDir)
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(JvmTarget.JVM_17)
    }
}

val bindgenKotlin = tasks.register<Exec>("bindgenKotlin") {
    description = "Generate Kotlin bindings from gemstone via uniffi"
    workingDir = gemstoneRoot
    inputs.dir(rustSrcDir)
    inputs.dir(cratesDir)
    inputs.file(gemstoneRoot.resolve("Cargo.toml"))
    outputs.dir(generatedKotlinDir.resolve("uniffi"))
    commandLine("just", "bindgen-kotlin")
}

val buildCargoNdk = tasks.register<Exec>("buildCargoNdk") {
    description = "Build gemstone native libraries using cargo-ndk"
    workingDir = gemstoneRoot
    inputs.dir(rustSrcDir)
    inputs.dir(cratesDir)
    inputs.file(gemstoneRoot.resolve("Cargo.toml"))
    outputs.dir(jniLibsDir)
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "armeabi-v7a",
        "-t", "x86_64",
        "-o", jniLibsDir.absolutePath,
        "build", "--lib"
    )
}

tasks.configureEach {
    if (name.matches(Regex("(compile|extract|source|javaDoc).*(Debug|Release).*"))) {
        dependsOn(bindgenKotlin)
    }
    if (name.matches(Regex("merge.*(Debug|Release).*JniLib.*"))) {
        dependsOn(buildCargoNdk)
    }
}

dependencies {
    api("net.java.dev.jna:jna:5.18.1@aar")

    implementation("androidx.core:core-ktx:1.17.0")

    androidTestImplementation("androidx.test.ext:junit:1.3.0")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.7.0")
}

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("release") {
                from(components["release"])
                groupId = "com.gemwallet.gemstone"
                artifactId = "gemstone"
                version = System.getenv("VER_NAME") ?: "1.0.0"
            }
            create<MavenPublication>("debug") {
                from(components["debug"])
                groupId = "com.gemwallet.gemstone"
                artifactId = "gemstone-debug"
                version = System.getenv("VER_NAME") ?: "1.0.0-debug"
            }
        }
    }
}
