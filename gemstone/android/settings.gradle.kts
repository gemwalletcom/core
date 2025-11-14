pluginManagement {
    repositories {
        val localProps = rootDir.resolve("local.properties").takeIf { it.isFile }?.inputStream()?.use { stream ->
            java.util.Properties().apply { load(stream) }
        }
        val githubUser = localProps?.getProperty("github.user")
        val githubToken = localProps?.getProperty("github.token")
        maven {
            url = uri("https://maven.pkg.github.com/0xh3rman/cargo-ndk-android-gradle")
            credentials {
                username = githubUser
                password = githubToken
            }
        }
        mavenLocal()
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        mavenLocal()
        google()
        mavenCentral()
    }
}

rootProject.name = "gemstone-android"
include(":gemstone")
