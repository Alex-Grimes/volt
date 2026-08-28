export interface FunctionHotspot {
  name: string;
  line: number;
  end_line: number;
  complexity: number;
  score: number;
}

export interface VoltResult {
  file_path: string;
  score: number;
  churn: number;
  complexity: number;
  functions?: FunctionHotspot[];
}

export type VoltageSeverity = 'high' | 'medium' | 'low' | 'minimal';

export interface VoltThresholds {
  high: number;
  medium: number;
  low: number;
}

export function getSeverity(score: number, thresholds: VoltThresholds): VoltageSeverity {
  if (score >= thresholds.high) {
    return 'high';
  }
  if (score >= thresholds.medium) {
    return 'medium';
  }
  if (score >= thresholds.low) {
    return 'low';
  }
  return 'minimal';
}
