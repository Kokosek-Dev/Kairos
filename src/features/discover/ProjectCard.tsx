import { motion } from "framer-motion";
import { Download, Heart, Package, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { formatBytes, formatCount } from "@/lib/format";
import type { Instance } from "@/lib/tauri/instances";
import type { SearchHit } from "@/lib/tauri/modrinth";
import { useJobsStore } from "./jobsStore";

interface ProjectCardProps {
  hit: SearchHit;
  instance: Instance | null;
}

export function ProjectIcon({ url, className }: { url: string | null; className?: string }) {
  return url ? (
    <img src={url} alt="" className={`shrink-0 rounded-lg bg-surface-raised object-cover ${className ?? "size-16"}`} draggable={false} />
  ) : (
    <div className={`grid shrink-0 place-items-center rounded-lg bg-surface-raised text-muted ${className ?? "size-16"}`}>
      <Package size={22} />
    </div>
  );
}

export function installBlocker(type: SearchHit["projectType"], instance: Instance | null): "noInstance" | "needsLoader" | null {
  if (type === "modpack") return null;
  if (!instance) return "noInstance";
  if (type === "mod" && instance.loader === "vanilla") return "needsLoader";
  return null;
}

export function InstallControl({ hit, instance }: ProjectCardProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const job = useJobsStore((state) => state.jobs[hit.projectId]);
  const { install, installModpack, cancel } = useJobsStore();
  const blocker = installBlocker(hit.projectType, instance);

  if (job) {
    const percent = job.totalBytes > 0 ? (job.doneBytes / job.totalBytes) * 100 : 0;
    return (
      <div className="w-44">
        <div className="h-1.5 overflow-hidden rounded-full bg-surface-raised">
          <div className="h-full rounded-full bg-accent transition-all" style={{ width: `${Math.max(percent, 4)}%` }} />
        </div>
        <div className="mt-1.5 flex items-center justify-between text-xs text-muted">
          <span>{job.totalBytes > 0 ? formatBytes(job.doneBytes) : t("versions.resolving")}</span>
          <button type="button" onClick={() => void cancel(hit.projectId)} aria-label={t("versions.cancel")}>
            <X size={14} />
          </button>
        </div>
      </div>
    );
  }

  const isPack = hit.projectType === "modpack";
  return (
    <Button
      size="sm"
      disabled={blocker !== null}
      title={blocker ? t(`discover.blocked.${blocker}`) : undefined}
      onClick={(event) => {
        event.stopPropagation();
        if (isPack) {
          void installModpack(t, hit.projectId).then((created) => created && navigate(`/instances/${created.id}`));
        } else if (instance) {
          void install(t, instance.id, hit.projectId);
        }
      }}
    >
      <Download size={14} />
      {isPack ? t("discover.installPack") : t("versions.install")}
    </Button>
  );
}

export function ProjectCard({ hit, instance }: ProjectCardProps) {
  const { i18n } = useTranslation();
  const navigate = useNavigate();
  const search = instance ? `?instance=${instance.id}` : "";

  return (
    <motion.article
      layout
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      onClick={() => navigate(`/discover/${hit.projectId}${search}`)}
      className="flex cursor-pointer items-center gap-4 rounded-xl border bg-surface p-4 transition-[border-color,box-shadow,background-color] hover:border-accent/40 hover:bg-surface-raised/60 hover:shadow-glow"
    >
      <ProjectIcon url={hit.iconUrl} />
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <h3 className="truncate font-semibold">{hit.title}</h3>
          <span className="shrink-0 text-xs text-muted">{hit.author}</span>
        </div>
        <p className="mt-0.5 line-clamp-2 text-muted">{hit.description}</p>
        <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted">
          <span className="flex items-center gap-1">
            <Download size={12} />
            {formatCount(hit.downloads, i18n.language)}
          </span>
          <span className="flex items-center gap-1">
            <Heart size={12} />
            {formatCount(hit.follows, i18n.language)}
          </span>
          {hit.displayCategories.slice(0, 4).map((category) => (
            <span key={category} className="rounded-full bg-surface-raised px-2 py-0.5 capitalize">
              {category}
            </span>
          ))}
        </div>
      </div>
      <div onClick={(event) => event.stopPropagation()}>
        <InstallControl hit={hit} instance={instance} />
      </div>
    </motion.article>
  );
}
