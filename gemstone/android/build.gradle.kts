plugins {
    id("org.jetbrains.kotlin.android") version "2.2.21" apply false
    id("com.android.application") version "8.13.1" apply false
    id("com.gemwallet.cargo-ndk") version "0.5.0" apply false
    id("io.github.gradle-nexus.publish-plugin") version "2.0.0"
}

// Must match the verified namespace on Maven Central
group = "com.gemwallet"

val localProps = rootDir.resolve("local.properties").takeIf { it.isFile }?.inputStream()?.use { stream ->
    java.util.Properties().apply { load(stream) }
}

val sonatypeUsername = localProps?.getProperty("sonatype.username") ?: System.getenv("SONATYPE_USERNAME")
val sonatypePassword = localProps?.getProperty("sonatype.password") ?: System.getenv("SONATYPE_PASSWORD")
val sonatypeStagingProfileId = localProps?.getProperty("sonatype.staging_profile_id") ?: System.getenv("SONATYPE_STAGING_PROFILE_ID")

nexusPublishing {
    packageGroup.set("com.gemwallet")
    repositories {
        sonatype {
            nexusUrl.set(uri("https://ossrh-staging-api.central.sonatype.com/service/local/"))
            snapshotRepositoryUrl.set(uri("https://central.sonatype.com/repository/maven-snapshots/"))
            username.set(sonatypeUsername)
            password.set(sonatypePassword)
            sonatypeStagingProfileId?.let { stagingProfileId.set(it) }
        }
    }
}
