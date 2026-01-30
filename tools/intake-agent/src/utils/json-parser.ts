/**
 * Robust JSON parsing utilities with streaming support and fallback.
 * Based on patterns from Taskmaster AI.
 */

import { JSONParser } from '@streamparser/json';

/**
 * Clean markdown code blocks from JSON text.
 */
export function cleanJsonText(text: string): string {
  return text
    .replace(/^```(?:json)?\s*\n?/i, '')
    .replace(/\n?```\s*$/i, '')
    .trim();
}

/**
 * Extract JSON object from response text, handling various formats.
 */
export function extractJsonObject<T>(text: string, key: string): T | null {
  const cleaned = cleanJsonText(text);
  
  try {
    const parsed = JSON.parse(cleaned);
    
    // If it has the expected key, return the value
    if (parsed && typeof parsed === 'object' && key in parsed) {
      return parsed[key] as T;
    }
    
    // If it's an array directly, return it
    if (Array.isArray(parsed)) {
      return parsed as T;
    }
    
    return null;
  } catch {
    // Try to find and extract JSON from the text
    return extractJsonFromText(cleaned, key);
  }
}

/**
 * Try to extract JSON from text that may contain prose.
 */
function extractJsonFromText<T>(text: string, key: string): T | null {
  // Look for JSON object with the key
  const keyPattern = new RegExp(`"${key}"\\s*:\\s*\\[`);
  const match = text.match(keyPattern);
  
  if (!match || match.index === undefined) {
    return null;
  }
  
  // Find the start of the JSON object
  let start = match.index;
  while (start > 0 && text[start - 1] !== '{') {
    start--;
  }
  if (start > 0) start--; // Include the {
  
  // Try to parse from this position
  const substring = text.slice(start);
  
  try {
    // Find matching braces
    let depth = 0;
    let end = 0;
    for (let i = 0; i < substring.length; i++) {
      if (substring[i] === '{') depth++;
      if (substring[i] === '}') depth--;
      if (depth === 0 && i > 0) {
        end = i + 1;
        break;
      }
    }
    
    if (end > 0) {
      const jsonStr = substring.slice(0, end);
      const parsed = JSON.parse(jsonStr);
      if (parsed && key in parsed) {
        return parsed[key] as T;
      }
    }
  } catch {
    // Continue to fallback
  }
  
  return null;
}

/**
 * Streaming JSON parser configuration.
 */
export interface StreamParserConfig<T> {
  /** JSON path to extract items from (e.g., '$.tasks.*') */
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
 */
export async function parseStreamingJson<T>(
  textStream: AsyncIterable<string>,
  config: StreamParserConfig<T>
): Promise<{ items: T[]; rawText: string }> {
  const items: T[] = [];
  let rawText = '';
  
  const parser = new JSONParser({ paths: [config.jsonPath] });
  
  parser.onValue = (value: { value: unknown }) => {
    const item = value.value || value;
    if (!config.itemValidator || config.itemValidator(item as T)) {
      items.push(item as T);
      config.onProgress?.(item as T, items.length);
    }
  };
  
  parser.onError = (error: Error) => {
    config.onError?.(error);
  };
  
  try {
    for await (const chunk of textStream) {
      rawText += chunk;
      parser.write(chunk);
    }
    parser.end();
  } catch (error) {
    // Streaming failed, try fallback
    config.onError?.(error instanceof Error ? error : new Error(String(error)));
  }
  
  // If streaming didn't get all items, try fallback parsing
  if (items.length === 0 && rawText) {
    const fallbackItems = extractJsonObject<T[]>(rawText, config.key);
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
  
  const items = extractJsonObject<T[]>(text, key);
  
  if (!items || !Array.isArray(items)) {
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
    typeof (item as { id: unknown }).id === 'number' &&
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
    typeof (item as { taskId: unknown }).taskId === 'number'
  );
}
