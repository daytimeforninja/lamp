plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.ksp)
    alias(libs.plugins.hilt)
}

android {
    namespace = "com.lamp.mobile"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.lamp.mobile"
        minSdk = 29
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
    }
}

dependencies {
    implementation(project(":core:model"))
    implementation(project(":core:database"))
    implementation(project(":core:network"))
    implementation(project(":core:data"))
    implementation(project(":core:contacts"))
    implementation(project(":core:common"))
    implementation(project(":feature:inbox"))
    implementation(project(":feature:nextactions"))
    implementation(project(":feature:projects"))
    implementation(project(":feature:waiting"))
    implementation(project(":feature:someday"))
    implementation(project(":feature:habits"))
    implementation(project(":feature:dailyplanning"))
    implementation(project(":feature:domode"))
    implementation(project(":feature:calendar"))
    implementation(project(":feature:notes"))
    implementation(project(":feature:lists"))
    implementation(project(":feature:contacts"))
    implementation(project(":feature:review"))
    implementation(project(":feature:settings"))
    implementation(project(":feature:conflicts"))
    implementation(project(":sync"))

    implementation(libs.core.ktx)
    implementation(libs.activity.compose)
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.ui)
    implementation(libs.compose.material3)
    implementation(libs.compose.material.icons)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.navigation.compose)
    implementation(libs.lifecycle.runtime)
    implementation(libs.lifecycle.viewmodel)
    implementation(libs.hilt.android)
    implementation(libs.hilt.navigation.compose)
    ksp(libs.hilt.compiler)

    debugImplementation(libs.compose.ui.tooling)

    testImplementation(libs.junit)
    testImplementation(libs.mockk)
    testImplementation(libs.coroutines.test)
}
