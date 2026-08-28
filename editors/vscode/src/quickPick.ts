import * as path from 'path';
import * as vscode from 'vscode';
import { VoltResult } from './types';

interface VoltQuickPickItem extends vscode.QuickPickItem {
  filePath: string;
  line?: number;
}

export async function showHotspotsQuickPick(results: VoltResult[], workspaceRoot?: string) {
  if (!results || results.length === 0) {
    vscode.window.showInformationMessage('Volt: No high voltage files found.');
    return;
  }

  const items: VoltQuickPickItem[] = [];

  for (const r of results) {
    items.push({
      label: `$(zap) ${r.file_path}`,
      description: `Score: ${r.score.toFixed(1)} (churn: ${r.churn}, complexity: ${r.complexity})`,
      detail: r.functions && r.functions.length > 0 ? `${r.functions.length} functions detected` : undefined,
      filePath: r.file_path,
    });

    if (r.functions) {
      for (const f of r.functions) {
        items.push({
          label: `    $(symbol-function) fn ${f.name}`,
          description: `L${f.line} | Score: ${f.score.toFixed(1)} (comp: ${f.complexity})`,
          detail: `  ↳ ${r.file_path}:${f.line}`,
          filePath: r.file_path,
          line: f.line,
        });
      }
    }
  }

  const selected = await vscode.window.showQuickPick(items, {
    placeHolder: '⚡ Select a high-voltage file or function to jump to',
    matchOnDescription: true,
    matchOnDetail: true,
  });

  if (selected) {
    const fullPath = workspaceRoot ? path.join(workspaceRoot, selected.filePath) : selected.filePath;
    const doc = await vscode.workspace.openTextDocument(vscode.Uri.file(fullPath));
    const editor = await vscode.window.showTextDocument(doc);

    if (selected.line && selected.line > 0) {
      const pos = new vscode.Position(selected.line - 1, 0);
      editor.selection = new vscode.Selection(pos, pos);
      editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
    }
  }
}
