import ReactMarkdown from "react-markdown";
import rehypeRaw from "rehype-raw";
import rehypeSanitize from "rehype-sanitize";
import remarkGfm from "remark-gfm";
import { openExternalLink } from "@/lib/tauri/modrinth";

export function Markdown({ children }: { children: string }) {
  return (
    <div className="prose-kairos select-text space-y-3 leading-relaxed [&_a]:text-accent [&_a]:underline-offset-2 hover:[&_a]:underline [&_code]:rounded [&_code]:bg-surface-raised [&_code]:px-1 [&_h1]:mt-5 [&_h1]:text-xl [&_h1]:font-bold [&_h2]:mt-4 [&_h2]:text-lg [&_h2]:font-semibold [&_h3]:font-semibold [&_img]:inline-block [&_img]:max-w-full [&_img]:rounded-md [&_li]:ml-5 [&_li]:list-disc [&_pre]:overflow-x-auto [&_pre]:rounded-md [&_pre]:bg-surface-raised [&_pre]:p-3 [&_table]:w-full [&_td]:border [&_td]:p-2 [&_th]:border [&_th]:p-2">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeRaw, rehypeSanitize]}
        components={{
          a: ({ href, children: label }) => (
            <a
              href={href}
              onClick={(event) => {
                event.preventDefault();
                if (href) void openExternalLink(href).catch(() => undefined);
              }}
            >
              {label}
            </a>
          ),
        }}
      >
        {children}
      </ReactMarkdown>
    </div>
  );
}
