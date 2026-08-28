package com.volt.statusbar

import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerEvent
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.CustomStatusBarWidget
import com.intellij.openapi.wm.StatusBar
import com.intellij.openapi.wm.StatusBarWidget
import com.intellij.openapi.wm.StatusBarWidgetFactory
import com.intellij.ui.components.JBLabel
import com.volt.model.VoltThresholds
import com.volt.service.VoltEventListener
import com.volt.service.VoltService
import java.awt.Component
import javax.swing.JComponent

class VoltStatusBarWidgetFactory : StatusBarWidgetFactory {
    override fun getId(): String = "VoltStatusBarWidget"
    override fun getDisplayName(): String = "Volt Hotspots Indicator"
    override fun isAvailable(project: Project): Boolean = true
    override fun createWidget(project: Project): StatusBarWidget = VoltStatusBarWidget(project)
    override fun disposeWidget(widget: StatusBarWidget) {
        widget.dispose()
    }
    override fun canBeEnabledOn(statusBar: StatusBar): Boolean = true
}

class VoltStatusBarWidget(private val project: Project) : CustomStatusBarWidget {
    private val label = JBLabel("⚡ Volt: --")

    init {
        updateLabel()

        project.messageBus.connect(this).subscribe(FileEditorManagerListener.FILE_EDITOR_MANAGER, object : FileEditorManagerListener {
            override fun selectionChanged(event: FileEditorManagerEvent) {
                updateLabel()
            }
        })

        VoltService.getInstance(project).addListener(object : VoltEventListener {
            override fun onScanFinished(results: List<com.volt.model.VoltResult>) {
                updateLabel()
            }
        })
    }

    private fun updateLabel() {
        val editor = FileEditorManager.getInstance(project).selectedEditor
        val file = editor?.file

        if (file == null) {
            label.text = "⚡ Volt: --"
            return
        }

        val service = VoltService.getInstance(project)
        val result = service.byPath[file.path] ?: service.byPath[file.name]

        if (result != null) {
            val severity = VoltThresholds.getSeverity(result.score)
            label.text = "⚡ Volt: ${"%.1f".format(result.score)} (${severity.name})"
            label.toolTipText = "File: ${result.file_path} | Churn: ${result.churn} | Complexity: ${result.complexity}"
        } else {
            label.text = "⚡ Volt: --"
            label.toolTipText = "File not in hotspot analysis"
        }
    }

    override fun ID(): String = "VoltStatusBarWidget"
    override fun install(statusBar: StatusBar) {}
    override fun dispose() {}
    override fun getComponent(): JComponent = label
}
