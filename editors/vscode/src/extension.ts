import * as path from 'path';
import * as vscode from 'vscode';
import { DecorationManager } from './decorations';
import { showHotspotsQuickPick } from './quickPick';
import { runVoltScan } from './runner';
import { VoltStatusBarItem } from './statusBar';
import { VoltTreeDataProvider } from './treeView';
import { VoltResult } from './types';

let cachedResults: VoltResult[] = [];
let resultsByPath: Map<string, VoltResult> = new Map();
let decorationManager: DecorationManager | undefined;
let treeDataProvider: VoltTreeDataProvider | undefined;
let statusBarItem: VoltStatusBarItem | undefined;

export function activate(context: vscode.ExtensionContext) {
  decorationManager = new DecorationManager();
  statusBarItem = new VoltStatusBarItem();

  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  treeDataProvider = new VoltTreeDataProvider(workspaceRoot);

  vscode.window.registerTreeDataProvider('volt.hotspotsView', treeDataProvider);

  const scanWorkspace = async (showNotification = false) => {
    const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!root) {
      if (showNotification) {
        vscode.window.showWarningMessage('Volt: No workspace folder open.');
      }
      return;
    }

    try {
      const results = await runVoltScan(root, context.globalStorageUri.fsPath);
      cachedResults = results;
      resultsByPath.clear();

      for (const r of results) {
        resultsByPath.set(r.file_path, r);
        const normalized = path.normalize(r.file_path);
        resultsByPath.set(normalized, r);
      }

      treeDataProvider?.setResults(results, root);

      if (vscode.window.activeTextEditor) {
        updateEditor(vscode.window.activeTextEditor);
      }

      if (showNotification) {
        vscode.window.showInformationMessage(`⚡ Volt: Scanned ${results.length} hotspot files.`);
      }
    } catch (err) {
      vscode.window.showErrorMessage(`⚡ Volt Error: ${(err as Error).message}`);
    }
  };

  const updateEditor = (editor: vscode.TextEditor) => {
    if (!editor || !editor.document) {
      statusBarItem?.hide();
      return;
    }

    const docPath = editor.document.uri.fsPath;
    const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    const relativePath = root ? path.relative(root, docPath) : docPath;

    const result = resultsByPath.get(relativePath) || resultsByPath.get(path.normalize(relativePath));

    decorationManager?.updateDecorations(editor, result);
    statusBarItem?.update(result);
  };

  // Commands
  context.subscriptions.push(
    vscode.commands.registerCommand('volt.scan', () => scanWorkspace(true)),
    vscode.commands.registerCommand('volt.refresh', () => scanWorkspace(false)),
    vscode.commands.registerCommand('volt.quickPick', () => {
      const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
      showHotspotsQuickPick(cachedResults, root);
    }),
    vscode.commands.registerCommand('volt.clear', () => {
      cachedResults = [];
      resultsByPath.clear();
      treeDataProvider?.setResults([]);
      if (vscode.window.activeTextEditor) {
        decorationManager?.clear(vscode.window.activeTextEditor);
      }
      statusBarItem?.hide();
      vscode.window.showInformationMessage('Volt: Cleared all hotspot annotations.');
    })
  );

  // Event listeners
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        updateEditor(editor);
      } else {
        statusBarItem?.hide();
      }
    }),
    vscode.workspace.onDidSaveTextDocument((doc) => {
      const config = vscode.workspace.getConfiguration('volt');
      if (config.get<boolean>('autoScan', true)) {
        scanWorkspace(false);
      }
    }),
    vscode.workspace.onDidChangeConfiguration((e) => {
      if (e.affectsConfiguration('volt')) {
        if (vscode.window.activeTextEditor) {
          updateEditor(vscode.window.activeTextEditor);
        }
      }
    })
  );

  // Initial scan if autoScan is enabled
  const config = vscode.workspace.getConfiguration('volt');
  if (config.get<boolean>('autoScan', true)) {
    scanWorkspace(false);
  }
}

export function deactivate() {
  decorationManager?.dispose();
  statusBarItem?.dispose();
}
