import { invoke } from "@tauri-apps/api/core";

/** A text for everybody, or one per language code. */
export type LocalizedText = string | Record<string, string>;

export interface NewsItem {
  id: string;
  kind: "news" | "sponsored";
  date: string | null;
  title: LocalizedText;
  body: LocalizedText;
  url: string | null;
}

export const fetchNews = () => invoke<NewsItem[]>("fetch_news");
