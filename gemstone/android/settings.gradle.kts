pluginManagement {
    repositories {
        val localProps = rootDir.resolve("local.properties").takeIf { it.isFile }?.inputStream()?.use { stream ->
            java.util.Properties().apply { load(stream) }
        }
        val githubUser = System.getenv("GITHUB_USER") ?: localProps?.getProperty("github.user")
        val githubToken = System.getenv("GITHUB_TOKEN") ?: localProps?.getProperty("github.token")
        maven {
            url = uri("https://maven.pkg.github.com/gemwalletcom/cargo-ndk-android-gradle")
            credentials {
                username = githubUser
                password = githubToken
            }
        }
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
        google()
        mavenCentral()
    }
}

rootProject.name = "gemstone-android"
include(":gemstone")
