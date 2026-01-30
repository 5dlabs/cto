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
    // Remove markdown code blocks
    .replace(/^```(?:json)?\s*\n?/gim, '')
    .replace(/\n?```\s*$/gim, '')
    // Remove leading/trailing whitespace
    .trim();
}

/**
 * Attempt to repair malformed JSON using jsonrepair library.
 * Handles common issues like:
 * - Missing closing brackets/braces
 * - Trailing commas
 * - Unquoted keys
 * - Single quotes instead of double
 */
export function repairJson(text: string): string {
  try {
    return jsonrepair(text);
  } catch {
    // If repair fails, return original
    return text;
  }
}

/**
 * Extract JSON array from text, handling various edge cases.
 * Tries multiple strategies in order of preference.
 */
export function extractJsonArray<T>(text: string, key?: string): T[] | null {
  const cleaned = cleanJsonText(text);
  
  // Strategy 1: Direct parse (fastest path)
  try {
    const parsed = JSON.parse(cleaned);
    if (key && typeof parsed === 'object' && parsed !== null && key in parsed) {
      const arr = parsed[key];
      if (Array.isArray(arr)) return arr as T[];
    }
    if (Array.isArray(parsed)) return parsed as T[];
  } catch {
    // Continue to repair strategies
  }

  // Strategy 2: Repair and parse
  try {
    const repaired = repairJson(cleaned);
    const parsed = JSON.parse(repaired);
    if (key && typeof parsed === 'object' && parsed !== null && key in parsed) {
      const arr = parsed[key];
      if (Array.isArray(arr)) return arr as T[];
    }
    if (Array.isArray(parsed)) return parsed as T[];
  } catch {
    // Continue to extraction strategies
  }

  // Strategy 3: Find and extract JSON object with key
  if (key) {
    const extracted = extractJsonObjectWithKey<T[]>(cleaned, key);
    if (extracted) return extracted;
  }

  // Strategy 4: Find array directly in text
  const arrayMatch = cleaned.match(/\[[\s\S]*\]/);
  if (arrayMatch) {
    try {
      const repaired = repairJson(arrayMatch[0]);
      const parsed = JSON.parse(repaired);
      if (Array.isArray(parsed)) return parsed as T[];
    } catch {
      // Continue to object extraction
    }
  }

  // Strategy 5: Extract individual objects from malformed array
  const objects = extractIndividualObjects<T>(cleaned);
  if (objects.length > 0) return objects;

  return null;
}

/**
 * Extract JSON object that contains a specific key.
 */
function extractJsonObjectWithKey<T>(text: string, key: string): T | null {
  // Find the key pattern
  const keyPattern = new RegExp(`["']?${key}["']?\\s*:\\s*\\[`);
  const match = text.match(keyPattern);
  
  if (!match || match.index === undefined) {
    return null;
  }

  // Find the opening brace before the key
  let start = match.index;
  while (start > 0 && text[start] !== '{') {
    start--;
  }

  // Extract from the opening brace to the end, then repair
  const substring = text.slice(start);
  
  try {
    const repaired = repairJson(substring);
    const parsed = JSON.parse(repaired);
    if (parsed && typeof parsed === 'object' && key in parsed) {
      return parsed[key] as T;
    }
  } catch {
    // Try finding the array directly after the key
    const arrayStart = match.index + match[0].length - 1; // Position of [
    const arraySubstring = text.slice(arrayStart);
    
    try {
      const repaired = repairJson(arraySubstring);
      const parsed = JSON.parse(repaired);
      if (Array.isArray(parsed)) return parsed as T;
    } catch {
      // Fall through
    }
  }

  return null;
}

/**
 * Extract individual JSON objects from text that may be a malformed array.
 * Useful when the array is incomplete but individual objects are valid.
 */
function extractIndividualObjects<T>(text: string): T[] {
  const objects: T[] = [];
  let depth = 0;
  let objectStart = -1;
  let inString = false;
  let escapeNext = false;

  for (let i = 0; i < text.length; i++) {
    const char = text[i];

    if (escapeNext) {
      escapeNext = false;
      continue;
    }

    if (char === '\\' && inString) {
      escapeNext = true;
      continue;
    }

    if (char === '"') {
      inString = !inString;
      continue;
    }

    if (inString) continue;

    if (char === '{') {
      if (depth === 0) {
        objectStart = i;
      }
      depth++;
    } else if (char === '}') {
      depth--;
      if (depth === 0 && objectStart !== -1) {
        const objectStr = text.slice(objectStart, i + 1);
        try {
          const repaired = repairJson(objectStr);
          const obj = JSON.parse(repaired);
          if (obj && typeof obj === 'object') {
            objects.push(obj as T);
          }
        } catch {
          // Skip malformed object
        }
        objectStart = -1;
      }
    }
  }

  // Try to parse any remaining incomplete object
  if (objectStart !== -1 && depth > 0) {
    const incompleteObj = text.slice(objectStart);
    try {
      const repaired = repairJson(incompleteObj);
      const obj = JSON.parse(repaired);
      if (obj && typeof obj === 'object') {
        objects.push(obj as T);
      }
    } catch {
      // Can't recover this object
    }
  }

  return objects;
}

/**
 * Extract JSON object from response text, handling various formats.
 * This is the main entry point for non-array JSON extraction.
 */
export function extractJsonObject<T>(text: string, key: string): T | null {
  const result = extractJsonArray<T>(text, key);
  return result as T | null;
}

/**
 * Streaming JSON parser configuration.
 */
export interface StreamParserConfig<T> {
  /** JSON path to extract items from (e.g., '$.tasks.*' or '$.*') */
  jsonPath: string;
  /** Key name in the response (e.g., 'tasks') */
  key: string;
  /** Validate each item */
  itemValidator?: (item: unknown) => item is T;
  /** Progress callback */
  onProgress?: (item: T, count: number) => void;
  /** Error callback */
  onError?: (error: Error) => void;
}

/**
 * Parse streaming text response with progress tracking.
 * Falls back to repair-based parsing if streaming fails.
 */
export async function parseStreamingJson<T>(
  textStream: AsyncIterable<string>,
  config: StreamParserConfig<T>
): Promise<{ items: T[]; rawText: string }> {
  const items: T[] = [];
  let rawText = '';
  let streamError: Error | null = null;
  
  const parser = new JSONParser({ paths: [config.jsonPath] });
  
  parser.onValue = (value: { value: unknown }) => {
    const item = value.value || value;
    if (!config.itemValidator || config.itemValidator(item as T)) {
      items.push(item as T);
      config.onProgress?.(item as T, items.length);
    }
  };
  
  parser.onError = (error: Error) => {
    streamError = error;
    config.onError?.(error);
  };
  
  try {
    for await (const chunk of textStream) {
      rawText += chunk;
      try {
        parser.write(chunk);
      } catch {
        // Streaming parse error, continue collecting text for fallback
      }
    }
    try {
      parser.end();
    } catch {
      // End error, use fallback
    }
  } catch (error) {
    streamError = error instanceof Error ? error : new Error(String(error));
    config.onError?.(streamError);
  }
  
  // If streaming got items, use them
  if (items.length > 0) {
    return { items, rawText };
  }

  // Fallback: Use repair-based extraction
  if (rawText) {
    const fallbackItems = extractJsonArray<T>(rawText, config.key);
    if (fallbackItems && Array.isArray(fallbackItems)) {
      for (const item of fallbackItems) {
        if (!config.itemValidator || config.itemValidator(item)) {
          items.push(item);
        }
      }
    }
  }
  
  return { items, rawText };
}

/**
 * Simple non-streaming JSON extraction with robust fallback.
 * Use this for single responses (not streaming).
 */
export function parseJsonResponse<T>(
  text: string,
  key: string,
  validator?: (item: unknown) => item is T
): { success: true; items: T[] } | { success: false; error: string } {
  if (!text || !text.trim()) {
    return { success: false, error: 'Empty response' };
  }
  
  const items = extractJsonArray<T>(text, key);
  
  if (!items || !Array.isArray(items) || items.length === 0) {
    // Last resort: try to extract individual objects without key
    const directObjects = extractIndividualObjects<T>(text);
    if (directObjects.length > 0) {
      if (validator) {
        const validItems = directObjects.filter(validator);
        if (validItems.length > 0) {
          return { success: true, items: validItems };
        }
      } else {
        return { success: true, items: directObjects };
      }
    }
    
    return { 
      success: false, 
      error: `Failed to extract ${key} array from response. Preview: ${text.slice(0, 200)}...` 
    };
  }
  
  // Validate items if validator provided
  if (validator) {
    const validItems = items.filter(validator);
    if (validItems.length !== items.length) {
      console.warn(`Filtered out ${items.length - validItems.length} invalid items`);
    }
    if (validItems.length === 0) {
      return { 
        success: false, 
        error: `All ${items.length} items failed validation` 
      };
    }
    return { success: true, items: validItems };
  }
  
  return { success: true, items };
}

/**
 * Validate a task object has required fields.
 */
export function isValidTask(item: unknown): item is { id: number; title: string } {
  return (
    typeof item === 'object' &&
    item !== null &&
    'id' in item &&
    (typeof (item as { id: unknown }).id === 'number' || typeof (item as { id: unknown }).id === 'string') &&
    'title' in item &&
    typeof (item as { title: unknown }).title === 'string'
  );
}

/**
 * Validate a subtask object has required fields.
 */
export function isValidSubtask(item: unknown): item is { id: number; title: string } {
  return isValidTask(item);
}

/**
 * Validate a complexity analysis object.
 */
export function isValidComplexityAnalysis(item: unknown): item is { taskId: number } {
  return (
    typeof item === 'object' &&
    item !== null &&
    'taskId' in item &&
    (typeof (item as { taskId: unknown }).taskId === 'number' || 
     typeof (item as { taskId: unknown }).taskId === 'string')
  );
}
