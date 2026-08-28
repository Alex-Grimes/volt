import * as vscode from 'vscode';
import { FunctionHotspot, getSeverity, VoltageSeverity, VoltResult, VoltThresholds } from './types';

export class DecorationManager {
  private decorationTypes: Map<VoltageSeverity, vscode.TextEditorDecorationType> = new Map();

  constructor() {
    this.initDecorationTypes();
  }

  private initDecorationTypes() {
    this.dispose();

    const colors: Record<VoltageSeverity, { color: string; overviewRulerColor: string }> = {
      high: {
        color: new vscode.ThemeColor('errorForeground').toString(),
        overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.errorForeground').toString(),
      },
      medium: {
        color: new vscode.ThemeColor('editorWarning.foreground').toString(),
        overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.warningForeground').toString(),
      },
      low: {
        color: new vscode.ThemeColor('editorInfo.foreground').toString(),
        overviewRulerColor: new vscode.ThemeColor('editorOverviewRuler.infoForeground').toString(),
      },
      minimal: {
        color: new vscode.ThemeColor('descriptionForeground').toString(),
        overviewRulerColor: 'transparent',
      },
    };

    for (const [sev, col] of Object.entries(colors) as [VoltageSeverity, { color: string; overviewRulerColor: string }][]) {
      const decType = vscode.window.createTextEditorDecorationType({
        overviewRulerLane: vscode.OverviewRulerLane.Right,
        overviewRulerColor: col.overviewRulerColor,
      });
      this.decorationTypes.set(sev, decType);
    }
  }

  public updateDecorations(editor: vscode.TextEditor, result?: VoltResult) {
    if (!editor || !editor.document) {
      return;
    }

    const config = vscode.workspace.getConfiguration('volt');
    const enabled = config.get<boolean>('decorations.enable', true);

    if (!enabled || !result) {
      this.clear(editor);
      return;
    }

    const thresholds: VoltThresholds = {
      high: config.get<number>('thresholds.high', 50),
      medium: config.get<number>('thresholds.medium', 20),
      low: config.get<number>('thresholds.low', 5),
    };

    const showFunctions = config.get<boolean>('decorations.showFunctions', true);
    const minFuncComplexity = config.get<number>('decorations.minFunctionComplexity', 5);

    const categorizedOptions: Map<VoltageSeverity, vscode.DecorationOptions[]> = new Map([
      ['high', []],
      ['medium', []],
      ['low', []],
      ['minimal', []],
    ]);

    // 1. File-level decoration at line 0
    const fileSeverity = getSeverity(result.score, thresholds);
    const fileHover = new vscode.MarkdownString(
      `### ⚡ Volt Hotspot Rating\n\n` +
      `- **Voltage Score**: \`${result.score.toFixed(1)}\`\n` +
      `- **Git Churn**: \`${result.churn} commits\`\n` +
      `- **AST Complexity**: \`${result.complexity}\`\n\n` +
      `*Formula: Churn × √Complexity*`
    );

    const fileRange = new vscode.Range(0, 0, 0, 0);
    categorizedOptions.get(fileSeverity)?.push({
      range: fileRange,
      hoverMessage: fileHover,
      renderOptions: {
        after: {
          contentText: `  ⚡ Voltage: ${result.score.toFixed(1)} (churn: ${result.churn}, complexity: ${result.complexity})`,
          fontStyle: 'italic',
        },
      },
    });

    // 2. Function-level decorations
    if (showFunctions && result.functions && result.functions.length > 0) {
      const lineCount = editor.document.lineCount;

      for (const func of result.functions) {
        if (func.complexity >= minFuncComplexity && func.line > 0) {
          const targetLine = Math.min(func.line - 1, lineCount - 1);
          if (targetLine === 0) {
            continue; // Don't collide with line 0 file header
          }

          const funcSeverity = getSeverity(func.score, thresholds);
          const funcHover = new vscode.MarkdownString(
            `### ⚡ Function Hotspot: \`${func.name}\`\n\n` +
            `- **Function Voltage**: \`${func.score.toFixed(1)}\`\n` +
            `- **Isolated Complexity**: \`${func.complexity}\`\n` +
            `- **Lines**: \`L${func.line} - L${func.end_line}\``
          );

          const funcRange = new vscode.Range(targetLine, 0, targetLine, 0);
          categorizedOptions.get(funcSeverity)?.push({
            range: funcRange,
            hoverMessage: funcHover,
            renderOptions: {
              after: {
                contentText: `  ⚡ fn ${func.name} (Voltage: ${func.score.toFixed(1)} | complexity: ${func.complexity})`,
                fontStyle: 'italic',
              },
            },
          });
        }
      }
    }

    // Apply decorations
    for (const [sev, options] of categorizedOptions.entries()) {
      const decType = this.decorationTypes.get(sev);
      if (decType) {
        editor.setDecorations(decType, options);
      }
    }
  }

  public clear(editor: vscode.TextEditor) {
    for (const decType of this.decorationTypes.values()) {
      editor.setDecorations(decType, []);
    }
  }

  public dispose() {
    for (const decType of this.decorationTypes.values()) {
      decType.dispose();
    }
    this.decorationTypes.clear();
  }
}
