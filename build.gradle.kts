import org.gradle.api.tasks.bundling.Jar
import org.gradle.api.tasks.bundling.Zip

plugins {
    `java-library`
    `maven-publish`
}

group = "io.github.lihao336991"
version = "0.2.1"

java {
    sourceCompatibility = JavaVersion.VERSION_1_8
    targetCompatibility = JavaVersion.VERSION_1_8
}

sourceSets {
    main {
        java.setSrcDirs(listOf("android/src/main/java"))
    }
}

val buildAndroidNative by tasks.registering(Exec::class) {
    inputs.dir("src")
    inputs.files("Cargo.toml", "Cargo.lock", "include/renderkit_resvg.h",
        "android/src/main/cpp/renderkit_resvg_jni.cpp", "build_android.py")
    outputs.dir(layout.buildDirectory.dir("android"))
    commandLine("python3", "build_android.py")
}

val sourcesJar by tasks.registering(Jar::class) {
    archiveClassifier.set("sources")
    from("android/src/main/java", "android/src/main/cpp", "src", "include")
}

val javadocJar by tasks.registering(Jar::class) {
    archiveClassifier.set("javadoc")
    dependsOn(tasks.javadoc)
    from(tasks.javadoc.map { it.destinationDir })
}

val androidAar by tasks.registering(Zip::class) {
    archiveBaseName.set("renderkit-resvg-android")
    archiveVersion.set(project.version.toString())
    archiveExtension.set("aar")
    destinationDirectory.set(layout.buildDirectory.dir("outputs/aar"))
    dependsOn(tasks.jar, buildAndroidNative)
    from("android/src/main/AndroidManifest.xml")
    from(tasks.jar.flatMap { it.archiveFile }) { rename { "classes.jar" } }
    from(layout.buildDirectory.dir("android/jni")) { into("jni") }
    from(layout.buildDirectory.dir("android/notices")) { into("assets/renderkit/resvg") }
    from(listOf("LICENSE-MIT", "LICENSE-APACHE", "NOTICE")) { into("META-INF") }
}

tasks.assemble { dependsOn(androidAar) }

publishing {
    publications {
        create<MavenPublication>("release") {
            artifactId = "renderkit-resvg-android"
            artifact(androidAar) { extension = "aar" }
            artifact(sourcesJar)
            artifact(javadocJar)
            pom {
                name.set("RenderKitResvg Android")
                description.set("A constrained, fail-closed Android JNI binding for resvg")
                url.set("https://github.com/lihao336991/renderkit-resvg")
                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                        distribution.set("repo")
                    }
                    license {
                        name.set("Apache License 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0.txt")
                        distribution.set("repo")
                    }
                }
                developers {
                    developer {
                        id.set("lihao336991")
                        name.set("LiHao")
                        email.set("lihao.zzz@bytedance.com")
                    }
                }
                scm {
                    connection.set("scm:git:https://github.com/lihao336991/renderkit-resvg.git")
                    developerConnection.set("scm:git:ssh://git@github.com/lihao336991/renderkit-resvg.git")
                    url.set("https://github.com/lihao336991/renderkit-resvg")
                    tag.set("v${project.version}")
                }
            }
        }
    }
    repositories {
        maven {
            name = "localRelease"
            url = uri(layout.buildDirectory.dir("repository"))
        }
    }
}
