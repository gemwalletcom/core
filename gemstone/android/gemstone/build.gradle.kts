import org.jetbrains.kotlin.gradle.dsl.JvmTarget

fun isSigningEnabled(): Boolean {
    val signingKey = System.getenv("MVN_SIGNING_KEY")
    val signingPassphrase = System.getenv("MVN_SIGNING_PASSPHRASE")
    return !signingKey.isNullOrBlank() && !signingPassphrase.isNullOrBlank()
}

plugins {
    id("com.android.library")
    id("maven-publish")
    id("signing")
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

tasks.matching {
    it.name.matches(Regex("(compile|extract|source).*(Debug|Release).*(Kotlin|Annotations|Jar)"))
}.configureEach {
    dependsOn(bindgenKotlin)
}
tasks.matching { it.name.matches(Regex("merge(Debug|Release)JniLibFolders")) }.configureEach {
    dependsOn(buildCargoNdk)
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
                pom {
                    name.set("Gemstone")
                    description.set("Gem Wallet Core Android library")
                    url.set("https://github.com/gemwalletcom/core")
                    licenses {
                        license {
                            name.set("MIT")
                            url.set("https://opensource.org/licenses/MIT")
                        }
                    }
                    scm {
                        url.set("https://github.com/gemwalletcom/core")
                    }
                    developers {
                        developer {
                            id.set("gemwallet")
                            name.set("Gem Wallet")
                        }
                    }
                }
            }
            create<MavenPublication>("debug") {
                from(components["debug"])
                groupId = "com.gemwallet.gemstone"
                artifactId = "gemstone-debug"
                version = System.getenv("VER_NAME") ?: "1.0.0-debug"
            }
        }
    }

    if (isSigningEnabled()) {
        signing {
            useInMemoryPgpKeys(System.getenv("MVN_SIGNING_KEY"), System.getenv("MVN_SIGNING_PASSPHRASE"))
            sign(publishing.publications["release"])
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
