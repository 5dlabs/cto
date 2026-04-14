import { useMemo, useState } from 'react'
import CodeMirror from '@uiw/react-codemirror'
import { markdown } from '@codemirror/lang-markdown'
import { oneDark } from '@codemirror/theme-one-dark'
import { EditorView } from '@codemirror/view'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Columns2, Eye, FileCode2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'

export type PrdEditorViewMode = 'split' | 'edit' | 'preview'

const VIEW_OPTIONS: { id: PrdEditorViewMode; label: string; icon: typeof Columns2 }[] = [
  { id: 'split', label: 'Split', icon: Columns2 },
  { id: 'edit', label: 'Markdown', icon: FileCode2 },
  { id: 'preview', label: 'Preview', icon: Eye },
]

const editorShellTheme = EditorView.theme(
  {
    '&': { height: '100%' },
    '.cm-scroller': { fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace' },
    '.cm-content': { paddingBlock: '12px' },
  },
  { dark: true }
)

type PrdMarkdownEditorProps = {
  value: string
  onChange: (next: string) => void
  className?: string
  /** Minimum height of the editor / preview region */
  minHeightPx?: number
}

export function PrdMarkdownEditor({
  value,
  onChange,
  className,
  minHeightPx = 400,
}: PrdMarkdownEditorProps) {
  const [mode, setMode] = useState<PrdEditorViewMode>('split')

  const extensions = useMemo(
    () => [markdown(), oneDark, editorShellTheme],
    []
  )

  const paneHeight = `max(${minHeightPx}px, min(58vh, 640px))`

  const editorSetup = {
    lineNumbers: true,
    foldGutter: true,
    highlightActiveLine: true,
  } as const

  return (
    <div className={cn('flex flex-col gap-3', className)}>
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-[11px] uppercase tracking-[0.2em] text-slate-500">View</span>
        <div className="flex rounded-[14px] border border-white/10 bg-black/25 p-0.5">
          {VIEW_OPTIONS.map(({ id, label, icon: Icon }) => {
            const active = mode === id
            return (
              <Button
                key={id}
                type="button"
                variant="ghost"
                size="sm"
                aria-pressed={active}
                onClick={() => setMode(id)}
                className={cn(
                  'h-8 gap-1.5 rounded-[12px] px-3 text-xs font-medium',
                  active
                    ? 'bg-cyan-400/15 text-cyan-50 shadow-[inset_0_0_0_1px_rgba(34,211,238,0.25)]'
                    : 'text-slate-400 hover:bg-white/[0.06] hover:text-slate-100'
                )}
              >
                <Icon className="h-3.5 w-3.5 opacity-90" />
                {label}
              </Button>
            )
          })}
        </div>
      </div>

      <div
        className="overflow-hidden rounded-[22px] border border-white/10 bg-[#0c1524] shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]"
        style={{ height: paneHeight }}
      >
        {mode === 'preview' ? (
          <ScrollArea className="h-full">
            <MarkdownPreviewBody markdown={value} />
          </ScrollArea>
        ) : mode === 'edit' ? (
          <div className="h-full min-h-0">
            <CodeMirror
              value={value}
              height="100%"
              theme="dark"
              extensions={extensions}
              onChange={onChange}
              className="text-[13px] [&_.cm-editor]:h-full [&_.cm-editor]:rounded-[22px] [&_.cm-editor]:outline-none"
              basicSetup={editorSetup}
            />
          </div>
        ) : (
          <div className="grid h-full min-h-0 grid-cols-1 grid-rows-2 lg:grid-cols-2 lg:grid-rows-1">
            <div className="flex min-h-0 min-w-0 flex-col border-b border-white/10 lg:border-b-0 lg:border-r">
              <CodeMirror
                value={value}
                height="100%"
                theme="dark"
                extensions={extensions}
                onChange={onChange}
                className="min-h-0 flex-1 text-[13px] [&_.cm-editor]:outline-none"
                basicSetup={editorSetup}
              />
            </div>
            <div className="min-h-0 min-w-0 overflow-hidden">
              <ScrollArea className="h-full">
                <MarkdownPreviewBody markdown={value} />
              </ScrollArea>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

function MarkdownPreviewBody({ markdown: md }: { markdown: string }) {
  return (
    <article
      className={cn(
        'prose prose-invert max-w-none px-5 py-4',
        'prose-sm lg:prose-base',
        'prose-headings:scroll-mt-4 prose-headings:font-semibold prose-headings:tracking-tight',
        'prose-h1:text-xl prose-h2:text-lg prose-h3:text-base',
        'prose-p:text-slate-300 prose-li:text-slate-300',
        'prose-strong:text-slate-100 prose-code:text-cyan-100',
        'prose-a:text-cyan-300/95 prose-a:no-underline hover:prose-a:underline',
        'prose-pre:bg-black/35 prose-pre:border prose-pre:border-white/10',
        'prose-hr:border-white/15'
      )}
    >
      {md.trim() ? (
        <ReactMarkdown remarkPlugins={[remarkGfm]}>{md}</ReactMarkdown>
      ) : (
        <p className="text-sm italic text-slate-500">Nothing to preview yet — start writing markdown.</p>
      )}
    </article>
  )
}
