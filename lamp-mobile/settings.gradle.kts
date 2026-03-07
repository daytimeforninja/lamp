pluginManagement {
    repositories {
        val mavenRepo = System.getenv("MAVEN_REPO")
        if (mavenRepo != null) {
            // Nix-managed local repo — Gradle never touches the network.
            maven { url = uri("file://$mavenRepo") }
        } else {
            // Fallback: only used inside the Nix FOD build to download deps.
            // Nix is the process connecting, not Gradle at dev time.
            mavenCentral()
            maven {
                url = uri("https://maven.google.com")
                content { includeGroupByRegex("com\\.android.*") }
                content { includeGroupByRegex("androidx.*") }
                content { includeGroupByRegex("com\\.google.*") }
            }
            gradlePluginPortal()
        }
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        val mavenRepo = System.getenv("MAVEN_REPO")
        if (mavenRepo != null) {
            maven { url = uri("file://$mavenRepo") }
        } else {
            mavenCentral()
            maven {
                url = uri("https://maven.google.com")
                content { includeGroupByRegex("com\\.android.*") }
                content { includeGroupByRegex("androidx.*") }
                content { includeGroupByRegex("com\\.google.*") }
            }
        }
    }
}

rootProject.name = "lamp-mobile"

include(":app")
include(":core:model")
include(":core:database")
include(":core:network")
include(":core:data")
include(":core:contacts")
include(":core:common")
include(":feature:inbox")
include(":feature:nextactions")
include(":feature:projects")
include(":feature:waiting")
include(":feature:someday")
include(":feature:habits")
include(":feature:dailyplanning")
include(":feature:domode")
include(":feature:calendar")
include(":feature:notes")
include(":feature:lists")
include(":feature:contacts")
include(":feature:review")
include(":feature:settings")
include(":feature:conflicts")
include(":sync")
