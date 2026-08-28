package com.volt.toolwindow

import com.intellij.openapi.fileEditor.OpenFileDescriptor
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.LocalFileSystem
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.treeStructure.Tree
import com.volt.model.FunctionHotspot
import com.volt.model.VoltResult
import com.volt.service.VoltEventListener
import com.volt.service.VoltService
import java.awt.BorderLayout
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import java.io.File
import javax.swing.JPanel
import javax.swing.tree.DefaultMutableTreeNode
import javax.swing.tree.DefaultTreeModel

class VoltToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val panel = JPanel(BorderLayout())
        val rootNode = DefaultMutableTreeNode("⚡ Volt Hotspots")
        val treeModel = DefaultTreeModel(rootNode)
        val tree = Tree(treeModel)
        tree.isRootVisible = true

        fun rebuildTree(results: List<VoltResult>) {
            rootNode.removeAllChildren()
            for (r in results) {
                val fileLabel = "${r.file_path} [Score: ${"%.1f".format(r.score)} | Churn: ${r.churn} | Comp: ${r.complexity}]"
                val fileNode = DefaultMutableTreeNode(FileNodeData(r, fileLabel))

                if (r.functions != null) {
                    for (f in r.functions) {
                        val funcLabel = "fn ${f.name} [L${f.line} | Score: ${"%.1f".format(f.score)} | Comp: ${f.complexity}]"
                        fileNode.add(DefaultMutableTreeNode(FuncNodeData(r, f, funcLabel)))
                    }
                }
                rootNode.add(fileNode)
            }
            treeModel.reload()
        }

        val service = VoltService.getInstance(project)
        rebuildTree(service.results)

        service.addListener(object : VoltEventListener {
            override fun onScanFinished(results: List<VoltResult>) {
                rebuildTree(results)
            }
        })

        tree.addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                if (e.clickCount == 2) {
                    val node = tree.lastSelectedPathComponent as? DefaultMutableTreeNode ?: return
                    val basePath = project.basePath ?: ""

                    when (val userObject = node.userObject) {
                        is FileNodeData -> {
                            val file = File(basePath, userObject.result.file_path)
                            val vFile = LocalFileSystem.getInstance().findFileByIoFile(file)
                            if (vFile != null) {
                                OpenFileDescriptor(project, vFile, 0, 0).navigate(true)
                            }
                        }
                        is FuncNodeData -> {
                            val file = File(basePath, userObject.fileResult.file_path)
                            val vFile = LocalFileSystem.getInstance().findFileByIoFile(file)
                            if (vFile != null) {
                                val targetLine = (userObject.func.line - 1).coerceAtLeast(0)
                                OpenFileDescriptor(project, vFile, targetLine, 0).navigate(true)
                            }
                        }
                    }
                }
            }
        })

        panel.add(JBScrollPane(tree), BorderLayout.CENTER)
        val content = ContentFactory.getInstance().createContent(panel, "", false)
        toolWindow.contentManager.addContent(content)
    }

    private data class FileNodeData(val result: VoltResult, val label: String) {
        override fun toString(): String = label
    }

    private data class FuncNodeData(val fileResult: VoltResult, val func: FunctionHotspot, val label: String) {
        override fun toString(): String = label
    }
}
