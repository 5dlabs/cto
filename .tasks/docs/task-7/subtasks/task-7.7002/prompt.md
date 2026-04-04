Implement subtask 7002: Integrate react-markdown for memo content rendering

## Objective
Install react-markdown and integrate it into the ResearchMemo component to render the `content` field as formatted markdown, including headers, links, and code blocks.

## Steps
1. Install `react-markdown` via npm/bun. 2. In the ResearchMemo CollapsibleContent area, render `<ReactMarkdown>{researchMemo.content}</ReactMarkdown>`. 3. Apply appropriate Tailwind prose classes (e.g., `prose prose-sm dark:prose-invert`) for consistent styling within the card context. 4. Verify that headers (h1-h3), inline code, code blocks, links, and lists render correctly. 5. Do NOT add remark/rehype plugins unless explicitly needed — keep it lightweight for v1.

## Validation
Component test: pass markdown string with h2 header, a link, and a code block as content. Verify rendered HTML contains <h2>, <a>, and <code>/<pre> elements respectively.