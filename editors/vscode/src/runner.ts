import * as cp from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import { VoltResult } from './types';

export function findBinaryPath(workspaceRoot?: string, globalStoragePath?: string): string | null {
  const config = vscode.workspace.getConfiguration('volt');
  const customPath = config.get<string>('binaryPath');

  if (customPath && customPath.trim().length > 0) {
    if (fs.existsSync(customPath)) {
      return customPath;
    }
  }

  const candidates: string[] = [];

  if (globalStoragePath) {
    const isWindows = process.platform === 'win32';
    candidates.push(
      path.join(globalStoragePath, isWindows ? 'volt-core.exe' : 'volt-core')
    );
  }

  if (workspaceRoot) {
    candidates.push(
      path.join(workspaceRoot, 'target', 'release', 'volt-core'),
      path.join(workspaceRoot, 'target', 'debug', 'volt-core'),
      path.join(workspaceRoot, 'bin', 'volt-core'),
      // In case workspace is in a subfolder of the repo
      path.join(workspaceRoot, '..', 'target', 'release', 'volt-core'),
      path.join(workspaceRoot, '..', 'target', 'debug', 'volt-core'),
      path.join(workspaceRoot, '..', '..', 'target', 'release', 'volt-core'),
      path.join(workspaceRoot, '..', '..', 'target', 'debug', 'volt-core')
    );
  }

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return 'volt-core';
}

export function runVoltScan(workspaceRoot: string, globalStoragePath?: string): Promise<VoltResult[]> {
  return new Promise((resolve, reject) => {
    const bin = findBinaryPath(workspaceRoot, globalStoragePath);

    if (!bin) {
      return reject(new Error('Could not locate volt-core binary. Build it with `cargo build --release` or configure `volt.binaryPath`.'));
    }

    cp.execFile(bin, [workspaceRoot], { maxBuffer: 10 * 1024 * 1024 }, (error, stdout, stderr) => {
      if (error) {
        return reject(new Error(stderr.trim() || error.message));
      }

      try {
        const results: VoltResult[] = JSON.parse(stdout);
        resolve(results);
      } catch (parseErr) {
        reject(new Error(`Failed to parse volt-core JSON output: ${(parseErr as Error).message}`));
      }
    });
  });
}
