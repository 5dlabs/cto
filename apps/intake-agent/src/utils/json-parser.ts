/**
 * Robust JSON parsing utilities with streaming support, repair, and fallback.
 * Based on patterns from Taskmaster AI with enhanced error recovery.
 */

import { JSONParser } from '@streamparser/json';
import { jsonrepair } from 'jsonrepair';

/**
 * Clean markdown code blocks and common formatting from JSON text.
 */
export function cleanJsonText(text: string): string {
  return text
    .replace(/^```(?:json)?\s*\n?/gim, '')
    .replace(/\n?```\s*$/gim, '')
    .trim();
}

/**
 * Attempt to repair malformed JSON using jsonrepair library.
 */
export function repairJson(text: string): string {
  try {
    return jsonrepair(text);
  } catch {
    return text;
  }
}

/**
 * Extract JSON array from text, handling various edge cases.
 */
export function extractJsonArray<T>(text: string, key?: string): T[] | null {
  const cleaned = cleanJsonText(text);
  
  // Strategy 1: Direct parse
  try {
    const parsed = JSON.parse(cleaned);
    if (key && typeof parsed === 'object' && parsed !== null && key in parsed) {
      const arr = parsed[key];
      if (Array.isArray(arr)) return arr as T[];
    }
    if (Array.isArray(parsed)) return parsed as T[];
  } catch {}

  // Strategy 2: Repair and parse
  try {
    const repaired = repairJson(cleaned);
    const parsed = JSON.parse(repaired);
    if (key && typeof parsed === 'object' && parsed !== null && key in parsed) {
      const arr = parsed[key];
      if (Array.isArray(arr)) return arr as T[];
    }
    if (Array.isArray(parsed)) return parsed as T[];
  } catch {}

  // Strategy 3: Find and extract JSON object with key
  if (key) {
    const extracted = extractJsonObjectWithKey<T[]>(cleaned, key);
    if (extracted) return extracted;
  }

  // Strategy 4: Find array directly
  const arrayMatch = cleaned.match(/\[[\s\S]*\]/);
  if (arrayMatch) {
    try {
      const repaired = repairJson(arrayMatch[0]);
      const parsed = JSON.parse(repaired);
      if (Array.isArray(parsed)) return parsed as T[];
    } catch {}
  }

  // Strategy 5: Extract individual objects
  const objects = extractIndividualObjects<T>(cleaned);
  if (objects.length > 0) return objects;

  return null;
}

function extractJsonObjectWithKey<T>(text: string, key: string): T | null {
  const keyPattern = new RegExp(`["']?${key}["']?\\s*:\\s*\\[`);
  const match = text.match(keyPattern);
  
  if (!match || match.index === undefined) return null;

  let start = match.index;
  while (start > 0 && text[start] !== '{') start--;

  const substring = text.slice(start);
  
  try {
    const repaired = repairJson(substring);
    const parsed = JSON.parse(repaired);
    if (parsed && typeof parsed === 'object' && key in parsed) {
      return parsed[key] as T;
    }
  } catch {}

  return null;
}

function extractIndividualObjects<T>(text: string): T[] {
  const objects: T[] = [];
  let depth = 0;
  let objectStart = -1;
  let inString = false;
  let escapeNext = false;

  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    if (escapeNext) { escapeNext = false; continue; }
    if (char === '\\' && inString) { escapeNext = true; continue; }
    if (char === '"') { inString = !inString; continue; }
    if (inString) continue;

    if (char === '{') {
      if (depth === 0) objectStart = i;
      depth++;
    } else if (char === '}') {
      depth--;
      if (depth === 0 && objectStart !== -1) {
        const objectStr = text.slice(objectStart, i + 1);
        try {
          const obj = JSON.parse(repairJson(objectStr));
          if (obj && typeof obj === 'object') objects.push(obj as T);
        } catch {}
        objectStart = -1;
      }
    }
  }

  if (objectStart !== -1 && depth > 0) {
    try {
      const obj = JSON.parse(repairJson(text.slice(objectStart)));
      if (obj && typeof obj === 'object') objects.push(obj as T);
    } catch {}
  }

  return objects;
}

export function extractJsonObject<T>(text: string, key: string): T | null {
  return extractJsonArray<T>(text, key) as T | null;
}

export interface StreamParserConfig<T> {
  jsonPath: string;
  key: string;
  itemValidator?: (item: unknown) => item is T;
  onProgress?: (item: T, count: number) => void;
  onError?: (error: Error) => void;
}

export async function parseStreamingJson<T>(
  textStream: AsyncIterable<string>,
  config: StreamParserConfig<T>
): Promise<{ items: T[]; rawText: string }> {
  const items: T[] = [];
  let rawText = '';
  
  const parser = new JSONParser({ paths: [config.jsonPath] });
  
  parser.onValue = (value: { value?: unknown }) => {
    const item = value.value ?? value;
    if (!config.itemValidator || config.itemValidator(item as T)) {
      items.push(item as T);
      config.onProgress?.(item as T, items.length);
    }
  };
  
  parser.onError = (error: Error) => config.onError?.(error);
  
  try {
    for await (const chunk of textStream) {
      rawText += chunk;
      try { parser.write(chunk); } catch {}
    }
    try { parser.end(); } catch {}
  } catch (error) {
    config.onError?.(error instanceof Error ? error : new Error(String(error)));
  }
  
  if (items.length === 0 && rawText) {
    const fallbackItems = extractJsonArray<T>(rawText, config.key);
    if (fallbackItems) {
      for (const item of fallbackItems) {
        if (!config.itemValidator || config.itemValidator(item)) items.push(item);
      }
    }
  }
  
  return { items, rawText };
}

export function parseJsonResponse<T>(
  text: string,
  key: string,
  validator?: (item: unknown) => item is T
): { success: true; items: T[] } | { success: false; error: string } {
  if (!text?.trim()) return { success: false, error: 'Empty response' };
  
  const items = extractJsonArray<T>(text, key);
  
  if (!items || items.length === 0) {
    const directObjects = extractIndividualObjects<T>(text);
    if (directObjects.length > 0) {
      const valid = validator ? directObjects.filter(validator) : directObjects;
      if (valid.length > 0) return { success: true, items: valid };
    }
    return { success: false, error: `Failed to extract ${key} array. Preview: ${text.slice(0, 200)}...` };
  }
  
  if (validator) {
    const validItems = items.filter(validator);
    if (validItems.length === 0) return { success: false, error: `All ${items.length} items failed validation` };
    return { success: true, items: validItems };
  }
  
  return { success: true, items };
}

export function isValidTask(item: unknown): item is { id: number; title: string } {
  return typeof item === 'object' && item !== null && 'id' in item && 'title' in item &&
    (typeof (item as any).id === 'number' || typeof (item as any).id === 'string') &&
    typeof (item as any).title === 'string';
}

export function isValidSubtask(item: unknown): item is { id: number; title: string } {
  return isValidTask(item);
}

export function isValidComplexityAnalysis(item: unknown): item is { taskId: number } {
  return typeof item === 'object' && item !== null && 'taskId' in item &&
    (typeof (item as any).taskId === 'number' || typeof (item as any).taskId === 'string');
}

// =============================================================================
// Single-Concern Validation
// =============================================================================

/**
 * Check if a subtask violates single-concern rule.
 * Returns true if violation detected.
 */
export function hasCombinedConcerns(title: string, details: string): boolean {
  const text = `${title} ${details}`.toLowerCase();
  
  // List of systems that should NOT be combined
  const systems = ['postgresql', 'mongodb', 'redis', 'kafka', 'rabbitmq', 'mysql', 'postgres'];
  
  // Find which systems are mentioned
  const found = systems.filter(s => text.includes(s));
  
  // Check for comma-separated lists (PostgreSQL, MongoDB, Redis)
  const commaListMatch = text.match(/\b(\w+), (\w+)\b/);
  if (commaListMatch && found.length >= 2) return true;
  
  // Check for "and" connecting multiple systems (Kafka and RabbitMQ)
  if (found.length >= 2 && text.includes(' and ')) return true;
  
  // Check for "(X, Y, Z)" pattern
  const parenListMatch = text.match(/\([^)]*\)/);
  if (parenListMatch) {
    const parenText = parenListMatch[0].toLowerCase();
    const parenSystems = systems.filter(s => parenText.includes(s));
    if (parenSystems.length >= 2) return true;
  }
  
  // Check for "namespaces, policies, quotas" pattern (multiple K8s concepts)
  const k8sPatterns = [
    /\bnamespaces?\b.*\bpolicies?\b/,
    /\bpolicies?\b.*\bquotas?\b/,
    /\bnamespaces?\b.*\bquotas?\b/,
    /\brbac\b.*\bnetwork policies?\b/,
  ];
  
  for (const pattern of k8sPatterns) {
    if (pattern.test(text)) return true;
  }
  
  // Count K8s-related words
  const k8sConcepts = ['namespaces', 'policies', 'quotas', 'rbac', 'network policy', 'security context'];
  const k8sFound = k8sConcepts.filter(c => text.includes(c));
  if (k8sFound.length >= 2) return true;
  
  return false;
}

/**
 * Validate that all subtasks follow single-concern rule.
 */
export function validateSingleConcern(subtasks: Array<{ id: number; title: string; details?: string }>): {
  valid: boolean;
  violations: Array<{ id: number; title: string; reason: string }>;
} {
  const violations: Array<{ id: number; title: string; reason: string }> = [];
  
  for (const subtask of subtasks) {
    const details = subtask.details || '';
    if (hasCombinedConcerns(subtask.title, details)) {
      violations.push({
        id: subtask.id,
        title: subtask.title,
        reason: 'Subtask combines multiple systems - split into separate subtasks'
      });
    }
  }
  
  return {
    valid: violations.length === 0,
    violations
  };
}
