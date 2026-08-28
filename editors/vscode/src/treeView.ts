import * as path from 'path';
import * as vscode from 'vscode';
import { FunctionHotspot, getSeverity, VoltResult, VoltThresholds } from './types';

export class VoltTreeItem extends vscode.TreeItem {
  constructor(
    public readonly label: string,
    public readonly collapsibleState: vscode.TreeItemCollapsibleState,
    public readonly fileResult?: VoltResult,
    public readonly funcHotspot?: FunctionHotspot,
    public readonly workspaceRoot?: string
  ) {
    super(label, collapsibleState);

    if (fileResult && !funcHotspot) {
      // File-level TreeItem
      this.description = `⚡ ${fileResult.score.toFixed(1)} (churn: ${fileResult.churn}, complexity: ${fileResult.complexity})`;
      this.tooltip = new vscode.MarkdownString(
        `### ${fileResult.file_path}\n\n` +
        `- **Score**: ${fileResult.score.toFixed(1)}\n` +
        `- **Churn**: ${fileResult.churn}\n` +
        `- **Complexity**: ${fileResult.complexity}\n` +
        `- **Functions**: ${fileResult.functions?.length || 0}`
      );

      const filePath = workspaceRoot
        ? path.join(workspaceRoot, fileResult.file_path)
        : fileResult.file_path;

      this.resourceUri = vscode.Uri.file(filePath);
      this.command = {
        command: 'vscode.open',
        title: 'Open File',
        arguments: [vscode.Uri.file(filePath)],
      };

      this.iconPath = new vscode.ThemeIcon('file-code');
    } else if (funcHotspot && fileResult) {
      // Function-level TreeItem
      this.description = `L${funcHotspot.line} | ⚡ ${funcHotspot.score.toFixed(1)} (comp: ${funcHotspot.complexity})`;
      this.tooltip = `Function: ${funcHotspot.name} (Lines ${funcHotspot.line}-${funcHotspot.end_line})`;

      const filePath = workspaceRoot
        ? path.join(workspaceRoot, fileResult.file_path)
        : fileResult.file_path;

      const targetPosition = new vscode.Position(Math.max(0, funcHotspot.line - 1), 0);
      this.command = {
        command: 'vscode.open',
        title: 'Go to Function',
        arguments: [
          vscode.Uri.file(filePath),
          {
            selection: new vscode.Range(targetPosition, targetPosition),
          },
        ],
      };

      this.iconPath = new vscode.ThemeIcon('symbol-function');
    }
  }
}

export class VoltTreeDataProvider implements vscode.TreeDataProvider<VoltTreeItem> {
  private _onDidChangeTreeData: vscode.EventEmitter<VoltTreeItem | undefined | null | void> =
    new vscode.EventEmitter<VoltTreeItem | undefined | null | void>();
  readonly onDidChangeTreeData: vscode.Event<VoltTreeItem | undefined | null | void> =
    this._onDidChangeTreeData.event;

  private results: VoltResult[] = [];
  private workspaceRoot?: string;

  constructor(workspaceRoot?: string) {
    this.workspaceRoot = workspaceRoot;
  }

  public setResults(results: VoltResult[], workspaceRoot?: string) {
    this.results = results;
    if (workspaceRoot) {
      this.workspaceRoot = workspaceRoot;
    }
    this._onDidChangeTreeData.fire();
  }

  public getTreeItem(element: VoltTreeItem): vscode.TreeItem {
    return element;
  }

  public getChildren(element?: VoltTreeItem): Thenable<VoltTreeItem[]> {
    if (!element) {
      // Root level: return all files sorted by voltage
      const items = this.results.map((r) => {
        const hasFunctions = (r.functions && r.functions.length > 0) || false;
        const collapsible = hasFunctions
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.None;

        return new VoltTreeItem(r.file_path, collapsible, r, undefined, this.workspaceRoot);
      });
      return Promise.resolve(items);
    }

    if (element.fileResult && !element.funcHotspot && element.fileResult.functions) {
      // Child level: return functions within this file
      const funcItems = element.fileResult.functions.map((f) => {
        return new VoltTreeItem(
          `fn ${f.name}`,
          vscode.TreeItemCollapsibleState.None,
          element.fileResult,
          f,
          this.workspaceRoot
        );
      });
      return Promise.resolve(funcItems);
    }

    return Promise.resolve([]);
  }
}
