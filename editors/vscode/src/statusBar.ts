import * as vscode from 'vscode';
import { getSeverity, VoltResult, VoltThresholds } from './types';

export class VoltStatusBarItem {
  private item: vscode.StatusBarItem;

  constructor() {
    this.item = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    this.item.command = 'volt.quickPick';
    this.item.tooltip = '⚡ Volt: Click to show hotspot files';
  }

  public update(result?: VoltResult) {
    if (!result) {
      this.item.text = '$(zap) Volt: --';
      this.item.tooltip = '⚡ Volt: File has not been scored yet';
      this.item.show();
      return;
    }

    const config = vscode.workspace.getConfiguration('volt');
    const thresholds: VoltThresholds = {
      high: config.get<number>('thresholds.high', 50),
      medium: config.get<number>('thresholds.medium', 20),
      low: config.get<number>('thresholds.low', 5),
    };

    const severity = getSeverity(result.score, thresholds);
    const label = severity.toUpperCase();

    this.item.text = `$(zap) Volt: ${result.score.toFixed(1)} (${label})`;
    this.item.tooltip = new vscode.MarkdownString(
      `### ⚡ ${result.file_path}\n\n` +
      `- **Voltage Score**: \`${result.score.toFixed(1)}\` (${label})\n` +
      `- **Churn**: \`${result.churn}\` revisions\n` +
      `- **AST Complexity**: \`${result.complexity}\`\n\n` +
      `*Click to open Volt Hotspots picker.*`
    );
    this.item.show();
  }

  public hide() {
    this.item.hide();
  }

  public dispose() {
    this.item.dispose();
  }
}
