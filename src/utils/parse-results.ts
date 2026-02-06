#!/usr/bin/env bun
/**
 * Parse Grok X Search results and save clean summaries with citations
 */

import fs from "fs/promises";
import path from "path";

const RESULTS_DIR = "./docs/grok-results";
const OUTPUT_DIR = "./docs/grok-summaries";

interface GrokResult {
  id: string;
  model: string;
  output: any[];
  citations?: string[];
}

// Extract text content from Grok response
function extractContent(output: any[]): string | null {
  for (const item of output || []) {
    if (item.content && typeof item.content === 'object') {
      for (const c of item.content) {
        if (c.type === 'output_text' && c.text && c.text.length > 50) {
          return c.text;
        }
      }
    }
  }
  return null;
}

// Extract citations from response
function extractCitations(output: any[]): string[] {
  const citations: string[] = [];
  
  for (const item of output || []) {
    if (item.annotations) {
      for (const ann of item.annotations) {
        if (ann.type === 'url_citation' && ann.url) {
          if (!citations.includes(ann.url)) {
            citations.push(ann.url);
          }
        }
      }
    }
  }
  
  return citations;
}

// Format citations as numbered list
function formatCitations(citations: string[]): string {
  if (citations.length === 0) return '';
  
  return citations.map((url, i) => `[${i + 1}]: ${url}`).join('\n');
}

// Parse all results
async function parseResults() {
  console.log('📂 Parsing Grok X Search Results...\n');
  
  await fs.mkdir(OUTPUT_DIR, { recursive: true });
  
  const categories = await fs.readdir(RESULTS_DIR).catch(() => []);
  
  for (const category of categories) {
    const catDir = path.join(RESULTS_DIR, category);
    const stat = await fs.stat(catDir);
    
    if (!stat.isDirectory()) continue;
    
    const files = (await fs.readdir(catDir))
      .filter(f => f.endsWith('.json'))
      .sort()
      .slice(-3); // Last 3 files per category
    
    if (files.length === 0) continue;
    
    console.log(`📁 ${category}: ${files.length} result(s)`);
    
    for (const file of files) {
      const filePath = path.join(catDir, file);
      const content = await fs.readFile(filePath, 'utf-8');
      const data: GrokResult = JSON.parse(content);
      
      const text = extractContent(data.output);
      const citations = extractCitations(data.output);
      
      if (!text) continue;
      
      // Save clean summary
      const timestamp = file.replace('.json', '');
      const outputFile = path.join(OUTPUT_DIR, `${category}-${timestamp}.md`);
      
      const summary = `# ${category}\n\n`;
      const dateStr = new Date().toISOString().split('T')[0];
      
      await fs.writeFile(outputFile, summary + `**Date:** ${dateStr}\n**Model:** ${data.model}\n**Response ID:** ${data.id}\n\n---\n\n## Summary\n\n${text}\n\n---\n\n## Citations\n\n${formatCitations(citations)}\n`);
      
      console.log(`  ✅ Saved: ${outputFile}`);
    }
  }
  
  console.log('\n✨ Done! Summaries saved to:', OUTPUT_DIR);
}

// Generate combined report
async function generateReport() {
  console.log('\n📊 Generating combined report...\n');
  
  const files = await fs.readdir(OUTPUT_DIR).catch(() => []);
  const summaries = files.filter(f => f.endsWith('.md')).sort();
  
  let report = `# CTO Research Report\n\n`;
  report += `Generated: ${new Date().toISOString()}\n\n`;
  report += `---\n\n`;
  
  for (const file of summaries) {
    const content = await fs.readFile(path.join(OUTPUT_DIR, file), 'utf-8');
    report += content + '\n\n---\n\n';
  }
  
  await fs.writeFile(path.join(OUTPUT_DIR, 'FULL-REPORT.md'), report);
  console.log('✅ Full report saved:', path.join(OUTPUT_DIR, 'FULL-REPORT.md'));
}

async function main() {
  await parseResults();
  await generateReport();
}

main().catch(console.error);
