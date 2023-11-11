plugins {
    kotlin("jvm") version "1.6.10" // Use the latest version
    id("com.github.johnrengelman.shadow") version "7.1.0" // For creating a fat JAR
}

group = "com.github.trino.querylog"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation(kotlin("stdlib"))
    implementation("io.trino:trino-spi:432") // Use the appropriate version of Trino SPI

    // Add any other dependencies your project needs here.
    // If you need to interact with native libraries, you might need to include JNI bindings.
    dependencies {
        // Jackson Kotlin module
        implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.13.0")
        implementation("com.fasterxml.jackson.datatype:jackson-datatype-jsr310:2.13.0")
        // Jackson's core and databind for data-binding
        implementation("com.fasterxml.jackson.core:jackson-databind:2.13.0")

        // Jackson annotations
        implementation("com.fasterxml.jackson.core:jackson-annotations:2.13.0")

        // Kotlin reflection, required for Jackson module
        implementation("org.jetbrains.kotlin:kotlin-reflect")

        implementation("ch.qos.logback:logback-classic:1.2.3") // Use the latest version
        implementation("org.slf4j:slf4j-api:1.7.30") // Use the latest version
    }


    testImplementation(kotlin("test"))
}

tasks.withType<Test> {
    useJUnitPlatform()
}

tasks.shadowJar {
    archiveClassifier.set("")
    manifest {
        attributes["Main-Class"] = "com.github.trino.querylog.QueryLogPlugin"
    }
}
