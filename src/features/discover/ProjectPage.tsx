import { useQuery } from "@tanstack/react-query";
import { ArrowLeft, Download, ExternalLink, Heart } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { Markdown } from "@/components/Markdown";
import { Button } from "@/components/ui/button";
import { useInstances } from "@/features/instances/useInstances";
import { formatBytes, formatCount, formatDate } from "@/lib/format";
import { errorMessage } from "@/lib/tauri/errors";
import { modrinthProject, modrinthVersions, openExternalLink, type ProjectVersion } from "@/lib/tauri/modrinth";
import { cn } from "@/lib/utils";
import { installBlocker, ProjectIcon } from "./ProjectCard";
import { useJobsStore } from "./jobsStore";

type Tab = "description" | "versions" | "gallery";

function VersionRow({ version, projectId, canInstall }: { version: ProjectVersion; projectId: string; canInstall: boolean }) {
  const { t, i18n } = useTranslation();
  const [params] = useSearchParams();
  const instanceId = params.get("instance");
  const install = useJobsStore((state) => state.install);
  const file = version.files.find((candidate) => candidate.primary) ?? version.files[0];

  return (
    <li className="flex items-center gap-4 rounded-lg border bg-surface px-4 py-3">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate font-medium">{version.name}</span>
          <span
            className={cn(
              "rounded-full px-2 py-0.5 text-[11px] font-medium",
              version.versionType === "release" ? "bg-success/15 text-success" : "bg-surface-raised text-muted",
            )}
          >
            {version.versionType}
          </span>
        </div>
        <p className="truncate text-xs text-muted">
          {version.versionNumber} · {version.loaders.join(", ")} · {version.gameVersions.slice(-3).join(", ")} ·{" "}
          {formatDate(version.datePublished, i18n.language)}
          {file ? ` · ${formatBytes(file.size)}` : ""}
        </p>
      </div>
      <Button
        size="sm"
        variant="secondary"
        disabled={!canInstall || !instanceId}
        onClick={() => instanceId && void install(t, instanceId, projectId, version.id)}
      >
        <Download size={14} />
        {t("versions.install")}
      </Button>
    </li>
  );
}

export function ProjectPage() {
  const { projectId = "" } = useParams();
  const { t, i18n } = useTranslation();
  const [params] = useSearchParams();
  const instances = useInstances();
  const instance = instances.data?.find((candidate) => candidate.id === params.get("instance")) ?? null;
  const [tab, setTab] = useState<Tab>("description");
  const [showAll, setShowAll] = useState(false);

  const details = useQuery({ queryKey: ["modrinth-project", projectId], queryFn: () => modrinthProject(projectId) });
  const project = details.data?.project;
  const filterByInstance = !showAll && instance !== null && project?.projectType !== "modpack";
  const versions = useQuery({
    queryKey: ["modrinth-versions", projectId, filterByInstance ? instance?.id : "all", instance?.gameVersion, instance?.loader],
    queryFn: () =>
      modrinthVersions(
        projectId,
        filterByInstance && project?.projectType === "mod" && instance && instance.loader !== "vanilla" ? instance.loader : null,
        filterByInstance && instance ? instance.gameVersion : null,
      ),
    enabled: tab === "versions" && project !== undefined,
  });
  const job = useJobsStore((state) => state.jobs[projectId]);
  const { install, installModpack } = useJobsStore();

  if (details.isPending) return <div className="skeleton h-96 max-w-4xl rounded-xl" />;
  if (details.isError || !project) {
    return (
      <div className="max-w-4xl space-y-3">
        <p className="text-danger">{errorMessage(t, details.error)}</p>
        <Button asChild variant="secondary">
          <Link to="/discover">{t("instances.back")}</Link>
        </Button>
      </div>
    );
  }

  const blocker = installBlocker(project.projectType, instance);
  const links = [
    [t("discover.links.source"), project.sourceUrl],
    [t("discover.links.issues"), project.issuesUrl],
    [t("discover.links.wiki"), project.wikiUrl],
    [t("discover.links.discord"), project.discordUrl],
  ].filter((link): link is [string, string] => link[1] !== null);

  return (
    <div className="max-w-4xl space-y-5 pb-10">
      <Link to={`/discover${instance ? `?instance=${instance.id}` : ""}`} className="inline-flex items-center gap-2 text-muted transition-colors hover:text-foreground">
        <ArrowLeft size={16} />
        {t("discover.back")}
      </Link>

      <header className="flex items-start gap-5">
        <ProjectIcon url={project.iconUrl} className="size-24" />
        <div className="min-w-0 flex-1">
          <h1 className="text-2xl font-bold tracking-tight">{project.title}</h1>
          <p className="mt-1 text-muted">{project.description}</p>
          <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted">
            <span className="flex items-center gap-1">
              <Download size={12} />
              {formatCount(project.downloads, i18n.language)}
            </span>
            <span className="flex items-center gap-1">
              <Heart size={12} />
              {formatCount(project.followers, i18n.language)}
            </span>
            {details.data.members[0] && <span>{t("discover.by", { name: details.data.members[0].user.username })}</span>}
            {project.license && <span>{project.license.name || project.license.id}</span>}
          </div>
        </div>
        <div className="flex flex-col items-end gap-2">
          <Button
            disabled={blocker !== null || job !== undefined}
            title={blocker ? t(`discover.blocked.${blocker}`) : undefined}
            onClick={() =>
              project.projectType === "modpack"
                ? void installModpack(t, project.id)
                : instance && void install(t, instance.id, project.id)
            }
          >
            <Download size={16} />
            {job
              ? t("versions.resolving")
              : project.projectType === "modpack"
                ? t("discover.installPack")
                : t("versions.install")}
          </Button>          {instance && project.projectType !== "modpack" && (
            <span className="text-xs text-muted">{t("discover.installingTo", { name: instance.name })}</span>
          )}
        </div>
      </header>

      <div className="flex gap-1 rounded-md bg-surface p-1 sm:w-fit">
        {(["description", "versions", "gallery"] as Tab[]).map((item) => (
          <button
            key={item}
            type="button"
            onClick={() => setTab(item)}
            className={cn(
              "h-8 rounded-sm px-4 text-sm font-medium transition-colors",
              tab === item ? "bg-accent text-accent-foreground" : "text-muted hover:text-foreground",
            )}
          >
            {t(`discover.tab.${item}`)}
          </button>
        ))}
      </div>

      {tab === "description" && (
        <div className="grid gap-6 lg:grid-cols-[1fr_14rem]">
          <section className="min-w-0 rounded-xl border bg-surface p-5">
            <Markdown>{project.body}</Markdown>
          </section>
          <aside className="space-y-4 text-sm">
            <div>
              <h3 className="mb-2 font-semibold">{t("discover.supports")}</h3>
              <p className="text-muted">{project.loaders.join(", ") || "-"}</p>
              <p className="mt-1 text-muted">{project.gameVersions.slice(-6).join(", ")}</p>
            </div>
            {links.length > 0 && (
              <div className="space-y-1.5">
                {links.map(([label, url]) => (
                  <button
                    key={label}
                    type="button"
                    onClick={() => void openExternalLink(url)}
                    className="flex items-center gap-2 text-accent hover:underline"
                  >
                    <ExternalLink size={13} />
                    {label}
                  </button>
                ))}
              </div>
            )}
          </aside>
        </div>
      )}

      {tab === "versions" && (
        <div className="space-y-3">
          {instance && project.projectType !== "modpack" && (
            <label className="flex items-center gap-2 text-sm text-muted">
              <input type="checkbox" checked={showAll} onChange={(event) => setShowAll(event.target.checked)} className="accent-accent" />
              {t("discover.showAllVersions")}
            </label>
          )}
          {versions.isPending && Array.from({ length: 4 }, (_, index) => <div key={index} className="skeleton h-16 rounded-lg" />)}
          {versions.isError && <p className="text-danger">{errorMessage(t, versions.error)}</p>}
          {versions.data?.length === 0 && <p className="py-6 text-center text-muted">{t("discover.noVersions")}</p>}
          <ul className="space-y-2">
            {versions.data?.map((version) => (
              <VersionRow key={version.id} version={version} projectId={project.id} canInstall={blocker === null && project.projectType !== "modpack"} />
            ))}
          </ul>
        </div>
      )}

      {tab === "gallery" && (
        <div className="grid gap-4 sm:grid-cols-2">
          {project.gallery.length === 0 && <p className="text-muted">{t("discover.noGallery")}</p>}
          {project.gallery.map((image) => (
            <figure key={image.url} className="overflow-hidden rounded-xl border bg-surface">
              <img src={image.url} alt={image.title ?? ""} className="w-full object-cover" loading="lazy" draggable={false} />
              {image.title && <figcaption className="p-3 text-sm text-muted">{image.title}</figcaption>}
            </figure>
          ))}
        </div>
      )}
    </div>
  );
}
