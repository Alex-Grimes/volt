package com.volt.runner

import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import com.intellij.openapi.project.Project
import com.volt.model.VoltResult
import java.io.File

object VoltRunner {
    private val gson = Gson()

    fun findBinaryPath(project: Project): String? {
        val basePath = project.basePath ?: return "volt-core"
        val candidates = listOf(
            File(basePath, "target/release/volt-core"),
            File(basePath, "target/debug/volt-core"),
            File(basePath, "bin/volt-core"),
            File(basePath, "../target/release/volt-core"),
            File(basePath, "../target/debug/volt-core"),
            File(basePath, "../../target/release/volt-core"),
            File(basePath, "../../target/debug/volt-core")
        )

        for (candidate in candidates) {
            if (candidate.exists() && candidate.canExecute()) {
                return candidate.absolutePath
            }
        }

        return "volt-core"
    }

    fun runScan(project: Project): List<VoltResult> {
        val basePath = project.basePath ?: return emptyList()
        val binary = findBinaryPath(project) ?: "volt-core"

        val process = ProcessBuilder(binary, basePath)
            .directory(File(basePath))
            .redirectErrorStream(false)
            .start()

        val output = process.inputStream.bufferedReader().readText()
        val exitCode = process.waitFor()

        if (exitCode != 0) {
            val error = process.errorStream.bufferedReader().readText()
            throw RuntimeException(if (error.isNotBlank()) error else "volt-core failed with exit code $exitCode")
        }

        val type = object : TypeToken<List<VoltResult>>() {}.type
        return gson.fromJson(output, type) ?: emptyList()
    }
}
