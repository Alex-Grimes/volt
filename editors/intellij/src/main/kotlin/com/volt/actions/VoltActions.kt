package com.volt.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.ui.Messages
import com.volt.service.VoltService

class ScanAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        VoltService.getInstance(project).scan { results ->
            Messages.showInfoMessage(project, "⚡ Volt: Scanned ${results.size} hotspot files.", "Volt Scan Complete")
        }
    }
}

class RefreshAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        VoltService.getInstance(project).scan()
    }
}

class ClearAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        VoltService.getInstance(project).clear()
    }
}
