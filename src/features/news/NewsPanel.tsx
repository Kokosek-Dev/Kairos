import { useQuery } from "@tanstack/react-query";
import { motion } from "framer-motion";
import { ChevronLeft, ChevronRight, ExternalLink, Newspaper } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { useIncognito } from "@/features/incognito/useIncognito";
import { usePreferences } from "@/features/settings/preferences";
import { openExternalLink } from "@/lib/tauri/modrinth";
import { fetchNews, type LocalizedText, type NewsItem } from "@/lib/tauri/news";
import { cn } from "@/lib/utils";

const OPEN_WIDTH = 288;
const CLOSED_WIDTH = 44;
const REFRESH_MS = 30 * 60 * 1000;

/** News that ship with the launcher, shown until (or instead of) a published list. */
/** What the latest updates brought, newest first. Every release adds its entry here and drops the oldest when the list passes six. */
const BUNDLED = ["studio", "bedrock", "feedback", "journal", "detective", "boost"] as const;

function pick(text: LocalizedText, language: string): string {
  if (typeof text === "string") return text;
  return text[language] ?? text[language.split("-")[0]] ?? text.en ?? Object.values(text)[0] ?? "";
}

function NewsCard({ item, language }: { item: NewsItem; language: string }) {
  const { t } = useTranslation();
  const sponsored = item.kind === "sponsored";
  return (
    <article className={cn("rounded-xl border bg-surface/70 p-3", sponsored && "border-dashed")}>
      <div className="mb-1 flex items-center justify-between gap-2 text-[11px] text-muted">
        <span className={cn("uppercase tracking-wider", sponsored && "font-semibold text-accent")}>{sponsored ? t("news.sponsored") : t("news.label")}</span>
        {item.date && <time dateTime={item.date}>{new Date(item.date).toLocaleDateString(language, { day: "numeric", month: "short" })}</time>}
      </div>
      <h3 className="font-semibold leading-snug">{pick(item.title, language)}</h3>
      <p className="mt-1 text-sm text-muted">{pick(item.body, language)}</p>
      {item.url && (
        <Button variant="ghost" size="sm" className="mt-2 -ml-2" onClick={() => void openExternalLink(item.url ?? "")}>
          <ExternalLink size={13} />
          {t("news.more")}
        </Button>
      )}
    </article>
  );
}

/** A collapsible column on the right with launcher news, with room for sponsored entries. */
export function NewsPanel() {
  const { t, i18n } = useTranslation();
  const open = usePreferences((state) => state.newsOpen);
  const update = usePreferences((state) => state.update);
  const incognito = useIncognito();
  const published = useQuery({
    queryKey: ["news"],
    queryFn: fetchNews,
    enabled: open && !incognito,
    staleTime: REFRESH_MS,
    refetchInterval: REFRESH_MS,
    retry: false,
  });

  const items: NewsItem[] = published.data?.length
    ? published.data
    : BUNDLED.map((key) => ({ id: key, kind: "news", date: null, title: t(`news.bundled.${key}.title`), body: t(`news.bundled.${key}.body`), url: null }));

  return (
    <motion.aside
      initial={false}
      animate={{ width: open ? OPEN_WIDTH : CLOSED_WIDTH }}
      transition={{ type: "spring", stiffness: 380, damping: 38 }}
      className="shrink-0 overflow-hidden border-l bg-surface/40 backdrop-blur-xl"
    >
      {open ? (
        <div className="flex h-full flex-col" style={{ width: OPEN_WIDTH }}>
          <div className="flex h-12 shrink-0 items-center justify-between border-b px-4">
            <h2 className="flex items-center gap-2 font-semibold">
              <Newspaper size={16} className="text-accent" />
              {t("news.title")}
            </h2>
            <Button variant="ghost" size="sm" className="size-8 px-0" aria-label={t("news.collapse")} title={t("news.collapse")} onClick={() => update({ newsOpen: false })}>
              <ChevronRight size={16} />
            </Button>
          </div>
          <div className="flex-1 space-y-3 overflow-y-auto p-3">
            {items.map((item) => (
              <NewsCard key={item.id} item={item} language={i18n.language} />
            ))}
          </div>
        </div>
      ) : (
        <button
          type="button"
          aria-label={t("news.expand")}
          title={t("news.expand")}
          onClick={() => update({ newsOpen: true })}
          className="flex h-full w-11 flex-col items-center gap-3 pt-3 text-muted transition-colors hover:text-foreground"
        >
          <ChevronLeft size={16} />
          <Newspaper size={16} className="text-accent" />
          <span className="text-xs font-medium [writing-mode:vertical-rl]">{t("news.title")}</span>
        </button>
      )}
    </motion.aside>
  );
}
