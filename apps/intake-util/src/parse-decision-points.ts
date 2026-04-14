/**
 * parse-decision-points — Extract DECISION_POINT blocks from debate turn text.
 *
 * Reads JSON from stdin: { content: string, speaker: "optimist" | "pessimist" }
 * Outputs JSON array of parsed decision points.
 */

const VALID_CATEGORIES = [
  'architecture',
  'error-handling',
  'data-model',
  'api-design',
  'ux-behavior',
  'performance',
  'security',
  'technology-choice',
  'infrastructure',
  'platform-choice',
  'build-vs-buy',
  'language-runtime',
  'service-topology',
  'visual-identity',
  'design-system',
  'component-library',
  'layout-pattern',
] as const;

type DPCategory = (typeof VALID_CATEGORIES)[number];

export interface ParsedDecisionPoint {
  id: string;
  category: DPCategory;
  question: string;
  proposing_option: string;
  reasoning: string;
  raised_by: 'optimist' | 'pessimist';
}

function isValidCategory(category: string): category is DPCategory {
  return (VALID_CATEGORIES as readonly string[]).includes(category);
}

export function parseDecisionPoints(
  content: string,
  speaker: 'optimist' | 'pessimist',
): ParsedDecisionPoint[] {
  const points: ParsedDecisionPoint[] = [];
  // Strip markdown bold/italic/code wrapping around DECISION_POINT labels
  // LLMs frequently emit **DECISION_POINT:** or *DECISION_POINT:* or `DECISION_POINT:`
  const cleaned = content.replace(/\*{1,2}(DECISION_POINT:)\*{1,2}/g, '$1')
                         .replace(/`(DECISION_POINT:)`/g, '$1');
  const blockRegex = /DECISION_POINT:\s*\n([\s\S]+?)(?=\n\nDECISION_POINT:|\n\n(?!\s)|$)/g;
  let blockMatch: RegExpExecArray | null;

  while ((blockMatch = blockRegex.exec(cleaned)) !== null) {
    const block = blockMatch[1] ?? '';
    const get = (field: string): string => {
      const m = new RegExp(`^${field}:\\s*(.+?)\\s*$`, 'm').exec(block);
      return m?.[1]?.trim() ?? '';
    };

    const id = get('id').replace(/[.,;:!?]+$/, '');
    const category = get('category').replace(/[.,;:!?]+$/, '');
    const question = get('question');
    const myOption = get('my_option');
    const reasoning =
      block.replace(/^(?:id|category|question|my_option):[^\n]*\n?/gm, '').trim() ||
      get('reasoning');

    if (!id || !category || !question || !myOption) continue;
    if (!isValidCategory(category)) continue;

    points.push({
      id,
      category,
      question,
      proposing_option: myOption,
      reasoning,
      raised_by: speaker,
    });
  }

  return points;
}
