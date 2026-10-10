import { Channel, invoke } from "@tauri-apps/api/core";
import type { Instance } from "./instances";
import type { InstallProgress } from "./versions";

export type ProjectType = "mod" | "modpack" | "resourcepack" | "shader";
export type ContentKind = "mod" | "resource_pack" | "shader";
export type SortIndex = "relevance" | "downloads" | "follows" | "newest" | "updated";

export interface SearchQuery {
  query: string;
  projectType: ProjectType;
  gameVersion: string | null;
  loader: string | null;
  categories: string[];
  sort: SortIndex;
  offset: number;
}

export interface SearchHit {
  projectId: string;
  slug: string;
  projectType: ProjectType;
  title: string;
  description: string;
  author: string;
  iconUrl: string | null;
  downloads: number;
  follows: number;
  displayCategories: string[];
  clientSide: string | null;
  serverSide: string | null;
  dateModified: string;
}

export interface SearchResults {
  hits: SearchHit[];
  offset: number;
  limit: number;
  totalHits: number;
}

export interface GalleryImage {
  url: string;
  title: string | null;
  featured: boolean;
}

export interface Project {
  id: string;
  slug: string;
  title: string;
  description: string;
  body: string;
  projectType: ProjectType;
  iconUrl: string | null;
  categories: string[];
  loaders: string[];
  gameVersions: string[];
  downloads: number;
  followers: number;
  clientSide: string | null;
  serverSide: string | null;
  license: { id: string; name: string } | null;
  issuesUrl: string | null;
  sourceUrl: string | null;
  wikiUrl: string | null;
  discordUrl: string | null;
  gallery: GalleryImage[];
}

export interface TeamMember {
  user: { username: string; avatarUrl: string | null };
  role: string;
}

export interface ProjectDetails {
  project: Project;
  members: TeamMember[];
}

export interface VersionFile {
  hashes: { sha1: string; sha512: string };
  url: string;
  filename: string;
  primary: boolean;
  size: number;
}

export interface ProjectVersion {
  id: string;
  projectId: string;
  name: string;
  versionNumber: string;
  changelog: string | null;
  versionType: "release" | "beta" | "alpha";
  gameVersions: string[];
  loaders: string[];
  datePublished: string;
  downloads: number;
  files: VersionFile[];
}

export interface Category {
  name: string;
  projectType: string;
  header: string;
}

export interface ContentEntry {
  kind: ContentKind;
  fileName: string;
  title: string;
  projectId: string | null;
  versionId: string | null;
  versionNumber: string | null;
  iconUrl: string | null;
  dependency: boolean;
}

export interface ContentItem {
  kind: ContentKind;
  fileName: string;
  enabled: boolean;
  size: number;
  entry: ContentEntry | null;
}

export interface InstallReport {
  installed: ContentEntry[];
  missingDependencies: string[];
}

export interface UpdateInfo {
  kind: ContentKind;
  fileName: string;
  title: string;
  projectId: string;
  currentVersion: string | null;
  newVersion: string;
  newVersionId: string;
}

export const modrinthSearch = (query: SearchQuery) => invoke<SearchResults>("modrinth_search", { query });
export const modrinthProject = (projectId: string) => invoke<ProjectDetails>("modrinth_project", { projectId });
export const modrinthVersions = (projectId: string, loader: string | null, gameVersion: string | null) =>
  invoke<ProjectVersion[]>("modrinth_versions", { projectId, loader, gameVersion });
export const modrinthCategories = () => invoke<Category[]>("modrinth_categories");

export const listContent = (instanceId: string) => invoke<ContentItem[]>("list_content", { instanceId });
export const setContentEnabled = (instanceId: string, kind: ContentKind, fileName: string, enabled: boolean) =>
  invoke<void>("set_content_enabled", { instanceId, kind, fileName, enabled });
export const removeContent = (instanceId: string, kind: ContentKind, fileName: string) =>
  invoke<void>("remove_content", { instanceId, kind, fileName });
export const checkContentUpdates = (instanceId: string) => invoke<UpdateInfo[]>("check_content_updates", { instanceId });

function withProgress<T>(command: string, args: Record<string, unknown>, onProgress: (p: InstallProgress) => void) {
  const channel = new Channel<InstallProgress>();
  channel.onmessage = onProgress;
  return invoke<T>(command, { ...args, onProgress: channel });
}

export const installContent = (
  instanceId: string,
  projectId: string,
  versionId: string | null,
  onProgress: (p: InstallProgress) => void,
) => withProgress<InstallReport>("install_content", { instanceId, projectId, versionId }, onProgress);

export const updateContent = (
  instanceId: string,
  updates: { projectId: string; versionId: string }[],
  onProgress: (p: InstallProgress) => void,
) => withProgress<InstallReport>("update_content", { instanceId, updates }, onProgress);

export const installModpack = (
  projectId: string,
  versionId: string | null,
  onProgress: (p: InstallProgress) => void,
) => withProgress<Instance>("install_modpack", { projectId, versionId }, onProgress);

export const importMrpack = (path: string, onProgress: (p: InstallProgress) => void) =>
  withProgress<Instance>("import_mrpack", { path }, onProgress);

export const exportMrpack = (instanceId: string, destination: string, name: string, version: string) =>
  invoke<void>("export_mrpack", { instanceId, destination, name, version });

export const pickMrpackFile = () => invoke<string | null>("pick_mrpack_file");
export const pickMrpackDestination = (fileName: string) =>
  invoke<string | null>("pick_mrpack_destination", { fileName });
export const getModrinthContact = () => invoke<string | null>("get_modrinth_contact");
export const setModrinthContact = (contact: string | null) => invoke<void>("set_modrinth_contact", { contact });
export const openExternalLink = (url: string) => invoke<void>("open_external_link", { url });
