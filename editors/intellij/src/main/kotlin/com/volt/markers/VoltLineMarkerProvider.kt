package com.volt.markers

import com.intellij.codeInsight.daemon.LineMarkerInfo
import com.intellij.codeInsight.daemon.LineMarkerProvider
import com.intellij.openapi.editor.markup.GutterIconRenderer
import com.intellij.openapi.util.IconLoader
import com.intellij.psi.PsiDocumentManager
import com.intellij.psi.PsiElement
import com.volt.service.VoltService
import javax.swing.Icon

class VoltLineMarkerProvider : LineMarkerProvider {
    private val zapIcon: Icon = IconLoader.getIcon("/icons/zap.svg", VoltLineMarkerProvider::class.java)

    override fun getLineMarkerInfo(element: PsiElement): LineMarkerInfo<*>? {
        val file = element.containingFile?.virtualFile ?: return null
        val project = element.project
        val service = VoltService.getInstance(project)
        val result = service.byPath[file.path] ?: service.byPath[file.name] ?: return null

        val document = PsiDocumentManager.getInstance(project).getDocument(element.containingFile) ?: return null
        val elementLine = document.getLineNumber(element.textOffset)

        // 1. File-level marker on first element of line 0
        if (elementLine == 0 && element.prevSibling == null) {
            val tooltip = """
                <html>
                <b>⚡ Volt Hotspot Rating</b><br/>
                • <b>Voltage Score</b>: ${"%.1f".format(result.score)}<br/>
                • <b>Git Churn</b>: ${result.churn} commits<br/>
                • <b>AST Complexity</b>: ${result.complexity}<br/>
                <i>Formula: Churn × √Complexity</i>
                </html>
            """.trimIndent()

            return LineMarkerInfo(
                element,
                element.textRange,
                zapIcon,
                { tooltip },
                null,
                GutterIconRenderer.Alignment.RIGHT,
                { "Volt File Hotspot" }
            )
        }

        // 2. Function-level markers
        if (result.functions != null) {
            for (func in result.functions) {
                if (func.complexity >= 5 && func.line > 0) {
                    val targetLine = func.line - 1
                    if (targetLine == elementLine && element.text == func.name) {
                        val tooltip = """
                            <html>
                            <b>⚡ Function Hotspot: <code>${func.name}</code></b><br/>
                            • <b>Function Voltage</b>: ${"%.1f".format(func.score)}<br/>
                            • <b>Complexity</b>: ${func.complexity}<br/>
                            • <b>Lines</b>: L${func.line} - L${func.end_line}
                            </html>
                        """.trimIndent()

                        return LineMarkerInfo(
                            element,
                            element.textRange,
                            zapIcon,
                            { tooltip },
                            null,
                            GutterIconRenderer.Alignment.RIGHT,
                            { "Volt Function Hotspot" }
                        )
                    }
                }
            }
        }

        return null
    }
}
