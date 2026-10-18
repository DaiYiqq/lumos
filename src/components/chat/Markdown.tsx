import hljs from "highlight.js";
import ReactMarkdown from "react-markdown";
import type { Components } from "react-markdown";
import remarkGfm from "remark-gfm";

interface MarkdownProps {
  /** Markdown 源文本。 */
  content: string;
  /** 点击引用编号时的回调。 */
  onCitation: (index: number) => void;
}

/**
 * 把正文里的 `[n]` 转成内部链接，交给自定义 `a` 渲染成可点击的上标。
 * 代码块与行内代码内的 `[n]` 不做转换。
 */
function withCitationLinks(content: string): string {
  return content
    .split(/(```[\s\S]*?```|`[^`]*`)/g)
    .map((part, index) => {
      if (index % 2 === 1) return part;
      return part.replace(/\[(\d{1,2})\]/g, (_match, number: string) => {
        return `[${number}](#citation-${number})`;
      });
    })
    .join("");
}

/** 渲染代码块，带 highlight.js 高亮。 */
function CodeBlock({ className, children }: { className?: string; children?: React.ReactNode }) {
  const language = className?.replace("language-", "") ?? "";
  const code = String(children ?? "").replace(/\n$/, "");

  // 高亮失败时回退为纯文本；try/catch 内只做计算，不构造 JSX（react-hooks 规则要求）。
  let highlighted: string | null = null;
  if (language && hljs.getLanguage(language)) {
    try {
      highlighted = hljs.highlight(code, { language, ignoreIllegals: true }).value;
    } catch {
      highlighted = null;
    }
  }

  return (
    <pre className="bg-surface-raised border-line my-3 overflow-x-auto rounded-lg border p-3 text-[12.5px] leading-relaxed">
      {highlighted ? (
        <code
          className={`hljs language-${language}`}
          dangerouslySetInnerHTML={{ __html: highlighted }}
        />
      ) : (
        <code>{code}</code>
      )}
    </pre>
  );
}

/** Markdown 渲染器：GFM 表格/任务列表 + 代码高亮 + 可点击引用。 */
export function Markdown({ content, onCitation }: MarkdownProps) {
  const components: Components = {
    p: ({ children }) => <p className="my-2 leading-relaxed">{children}</p>,
    h1: ({ children }) => <h1 className="mt-4 mb-2 text-lg font-semibold">{children}</h1>,
    h2: ({ children }) => <h2 className="mt-4 mb-2 text-base font-semibold">{children}</h2>,
    h3: ({ children }) => <h3 className="mt-3 mb-1.5 text-[15px] font-semibold">{children}</h3>,
    ul: ({ children }) => <ul className="my-2 list-disc space-y-1 pl-5">{children}</ul>,
    ol: ({ children }) => <ol className="my-2 list-decimal space-y-1 pl-5">{children}</ol>,
    li: ({ children }) => <li className="leading-relaxed">{children}</li>,
    blockquote: ({ children }) => (
      <blockquote className="border-brand/40 text-ink-muted my-3 border-l-2 pl-3">
        {children}
      </blockquote>
    ),
    table: ({ children }) => (
      <div className="my-3 overflow-x-auto">
        <table className="border-line w-full border-collapse text-[13px]">{children}</table>
      </div>
    ),
    th: ({ children }) => <th className="border-line border px-2 py-1 text-left">{children}</th>,
    td: ({ children }) => <td className="border-line border px-2 py-1">{children}</td>,
    hr: () => <hr className="border-line my-4" />,
    a: ({ href, children }) => {
      if (href?.startsWith("#citation-")) {
        const index = Number.parseInt(href.replace("#citation-", ""), 10);
        return (
          <button
            type="button"
            onClick={() => {
              if (Number.isFinite(index)) onCitation(index);
            }}
            className="bg-brand-soft text-brand hover:bg-brand mx-0.5 inline-flex h-[17px] min-w-[17px] -translate-y-0.5 items-center justify-center rounded px-1 align-baseline text-[10px] font-semibold transition-colors hover:text-white"
            aria-label={`查看引用 ${index}`}
          >
            {index}
          </button>
        );
      }
      return (
        <a
          href={href}
          target="_blank"
          rel="noreferrer"
          className="text-brand underline underline-offset-2"
        >
          {children}
        </a>
      );
    },
    code: ({ className, children, ...props }) => {
      const isBlock = Boolean(className) || String(children ?? "").includes("\n");
      if (isBlock) {
        return <CodeBlock className={className}>{children}</CodeBlock>;
      }
      return (
        <code
          className="bg-surface-raised border-line rounded border px-1 py-0.5 font-mono text-[12.5px]"
          {...props}
        >
          {children}
        </code>
      );
    },
  };

  return (
    <div className="text-[13.5px]">
      <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
        {withCitationLinks(content)}
      </ReactMarkdown>
    </div>
  );
}
