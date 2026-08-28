package com.volt.service

import com.intellij.codeInsight.daemon.DaemonCodeAnalyzer
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.ProjectActivity
import com.volt.model.VoltResult
import com.volt.runner.VoltRunner
import java.io.File
import java.util.concurrent.CopyOnWriteArrayList

interface VoltEventListener {
    fun onScanFinished(results: List<VoltResult>)
}

@Service(Service.Level.PROJECT)
class VoltService(private val project: Project) {
    var results: List<VoltResult> = emptyList()
        private set
    var byPath: Map<String, VoltResult> = emptyMap()
        private set

    private val listeners = CopyOnWriteArrayList<VoltEventListener>()

    fun addListener(listener: VoltEventListener) {
        listeners.add(listener)
    }

    fun removeListener(listener: VoltEventListener) {
        listeners.remove(listener)
    }

    fun scan(onFinished: ((List<VoltResult>) -> Unit)? = null) {
        ProgressManager.getInstance().run(object : Task.Backgroundable(project, "⚡ Volt: Scanning Hotspots...", true) {
            override fun run(indicator: ProgressIndicator) {
                try {
                    indicator.isIndeterminate = true
                    val scanResults = VoltRunner.runScan(project)
                    val map = mutableMapOf<String, VoltResult>()
                    val basePath = project.basePath ?: ""

                    for (r in scanResults) {
                        map[r.file_path] = r
                        map[File(basePath, r.file_path).absolutePath] = r
                    }

                    results = scanResults
                    byPath = map

                    ApplicationManager.getApplication().invokeLater {
                        DaemonCodeAnalyzer.getInstance(project).restart()
                        for (listener in listeners) {
                            listener.onScanFinished(scanResults)
                        }
                        onFinished?.invoke(scanResults)
                    }
                } catch (e: Exception) {
                    println("Volt scan error: ${e.message}")
                }
            }
        })
    }

    fun clear() {
        results = emptyList()
        byPath = emptyMap()
        ApplicationManager.getApplication().invokeLater {
            DaemonCodeAnalyzer.getInstance(project).restart()
            for (listener in listeners) {
                listener.onScanFinished(emptyList())
            }
        }
    }

    companion object {
        fun getInstance(project: Project): VoltService = project.service()
    }
}

class VoltStartupActivity : ProjectActivity {
    override suspend fun execute(project: Project) {
        VoltService.getInstance(project).scan()
    }
}
