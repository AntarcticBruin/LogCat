import type {
  HighlightSegment,
  HighlightTone,
  HighlightedLine,
} from "../types/app";

const highlightMatchers: Array<{ pattern: RegExp; tone: HighlightTone }> = [
  { pattern: /\b(fatal|panic|error|err|exception|traceback|failed|failure)\b/gi, tone: "error" },
  { pattern: /\b(warn|warning)\b/gi, tone: "warn" },
  { pattern: /\b(info|notice)\b/gi, tone: "info" },
  { pattern: /\b(debug)\b/gi, tone: "debug" },
  { pattern: /\b(trace|verbose)\b/gi, tone: "trace" },
  { pattern: /\b(ok|success|succeeded|started|connected|ready|listening)\b/gi, tone: "success" },
  { pattern: /\b\d{4}-\d{2}-\d{2}[ t]\d{2}:\d{2}:\d{2}(?:[.,]\d+)?(?:z|[+-]\d{2}:?\d{2})?\b/gi, tone: "muted" },
  { pattern: /\b(?:[1-5]\d{2})\b/g, tone: "info" },
];

export function detectLineTone(line: string): HighlightTone {
  if (/\b(fatal|panic|error|err|exception|traceback|failed|failure)\b/i.test(line)) return "error";
  if (/\b(warn|warning)\b/i.test(line)) return "warn";
  if (/\b(ok|success|succeeded|started|connected|ready|listening)\b/i.test(line)) return "success";
  if (/\b(info|notice)\b/i.test(line)) return "info";
  if (/\b(debug)\b/i.test(line)) return "debug";
  if (/\b(trace|verbose)\b/i.test(line)) return "trace";
  return "default";
}

export function buildSegments(line: string): HighlightSegment[] {
  if (!line.length) {
    return [{ text: "", tone: "default" }];
  }

  type MatchRecord = { start: number; end: number; tone: HighlightTone };
  const matches: MatchRecord[] = [];

  // 1. 收集所有匹配项
  for (const matcher of highlightMatchers) {
    matcher.pattern.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = matcher.pattern.exec(line)) !== null) {
      matches.push({
        start: match.index,
        end: match.index + match[0].length,
        tone: matcher.tone,
      });
      if (match[0].length === 0) matcher.pattern.lastIndex += 1;
    }
  }

  // 2. 按起始位置排序
  matches.sort((a, b) => a.start - b.start);

  // 3. 线性扫描构建分段
  const segments: HighlightSegment[] = [];
  let lastIndex = 0;

  for (const m of matches) {
    // 处理两个匹配项之间的普通文本
    if (m.start > lastIndex) {
      segments.push({ text: line.substring(lastIndex, m.start), tone: "default" });
    }
    // 确保不重复处理已覆盖的区域（处理重叠正则）
    if (m.end > lastIndex) {
      const actualStart = Math.max(m.start, lastIndex);
      segments.push({ text: line.substring(actualStart, m.end), tone: m.tone });
      lastIndex = m.end;
    }
  }

  // 处理剩余部分
  if (lastIndex < line.length) {
    segments.push({ text: line.substring(lastIndex), tone: "default" });
  }

  return segments.length > 0 ? segments : [{ text: line, tone: "default" }];
}

export function highlightLogLine(line: string): HighlightedLine {
  return {
    tone: detectLineTone(line),
    segments: buildSegments(line),
  };
}

export function highlightLogContent(content: string): HighlightedLine[] {
  return content.split("\n").map(highlightLogLine);
}
