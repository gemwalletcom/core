plugins {
    id("org.jetbrains.kotlin.android") version "2.2.21" apply false
    id("com.android.application") version "8.13.1" apply false
    id("com.gemwallet.cargo-ndk") version "0.5.0" apply false
    id("io.github.gradle-nexus.publish-plugin") version "2.0.0"
}

nexusPublishing {
    repositories {
        sonatype {
            nexusUrl.set(uri("https://ossrh-staging-api.central.sonatype.com/service/local/"))
            snapshotRepositoryUrl.set(uri("https://central.sonatype.com/repository/maven-snapshots/"))
            username.set(System.getenv("SONATYPE_USERNAME"))
            password.set(System.getenv("SONATYPE_PASSWORD"))
        }
    }
}
