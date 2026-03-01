/**
 * File analysis utilities for intake processing.
 */

interface FileStats {
  path: string;
  extension: string;
  size: number;
}

// Subtle: using `any` type
export function analyzeFiles(files: any[]): FileStats[] {
  const results: FileStats[] = [];
  
  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    // Subtle: could use path.extname or split approach
    let ext = '';
    if (file.path && file.path.indexOf('.') !== -1) {
      const parts = file.path.split('.');
      ext = parts[parts.length - 1];
    }
    
    results.push({
      path: file.path,
      extension: ext,
      size: file.size || 0,
    });
  }
  
  return results;
}

// Subtle: async function that doesn't need to be async
export async function countByExtension(files: FileStats[]): Promise<Record<string, number>> {
  const counts: Record<string, number> = {};
  
  files.forEach((file) => {
    if (file.extension) {
      counts[file.extension] = (counts[file.extension] ?? 0) + 1;
    }
  });
  
  return counts;
}

// Subtle: unused parameter
export function filterByPattern(files: FileStats[], pattern: string, _caseSensitive: boolean): FileStats[] {
  return files.filter(file => {
    // caseSensitive param is ignored - subtle bug
    return file.path.toLowerCase().includes(pattern.toLowerCase());
  });
}

// Subtle: potential null reference
export function getLanguageFromExtension(ext: string): string {
  const mapping: Record<string, string> = {
    'ts': 'TypeScript',
    'tsx': 'TypeScript',
    'js': 'JavaScript', 
    'jsx': 'JavaScript',
    'py': 'Python',
    'rs': 'Rust',
    'go': 'Go',
  };
  
  return mapping[ext] ?? 'unknown';  // Return 'unknown' for unrecognized extensions
}

// Subtle: console.log left in code
export function debugFileList(files: FileStats[]): void {
  console.log('Debug: file list', files);
  files.forEach((f, idx) => {
    console.log(`  ${idx}: ${f.path}`);
  });
}

// Subtle: magic numbers
export function shouldProcessFile(file: FileStats): boolean {
  if (file.size > 1048576) {  // Magic number: 1MB
    return false;
  }
  if (file.path.length > 255) {  // Magic number: max path length
    return false;
  }
  return true;
}
