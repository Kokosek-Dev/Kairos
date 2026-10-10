import { motion } from "framer-motion";
import { Check, Download, Play, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { formatBytes, formatDate } from "@/lib/format";
import type { InstallProgress, ManifestEntry } from "@/lib/tauri/versions";
import { cn } from "@/lib/utils";

interface VersionRowProps {
  entry: ManifestEntry;
  installed: boolean;
  progress?: InstallProgress;
  error?: string;
  playDisabled: boolean;
  onPlay: () => void;
  onInstall: () => void;
  onCancel: () => void;
}

export function VersionRow({
  entry,
  installed,
  progress,
  error,
  playDisabled,
  onPlay,
  onInstall,
  onCancel,
}: VersionRowProps) {
  const { t, i18n } = useTranslation();
  const percent = progress && progress.totalBytes > 0 ? (progress.doneBytes / progress.totalBytes) * 100 : 0;

  return (
    <div className="rounded-lg border bg-surface px-4 py-3 transition-colors hover:bg-surface-raised/60 [content-visibility:auto] [contain-intrinsic-size:auto_60px]">
      <div className="flex items-center gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="font-semibold">{entry.id}</span>
            <span
              className={cn(
                "rounded-full px-2 py-0.5 text-[11px] font-medium",
                entry.type === "release" ? "bg-accent/15 text-accent" : "bg-surface-raised text-muted",
              )}
            >
              {t(`versions.kind.${entry.type}`)}
            </span>
          </div>
          <p className="text-xs text-muted">{formatDate(entry.releaseTime, i18n.language)}</p>
        </div>

        {progress ? (
          <Button variant="ghost" size="sm" onClick={onCancel}>
            <X size={14} />
            {t("versions.cancel")}
          </Button>
        ) : installed ? (
          <div className="flex items-center gap-3">
            <span className="flex items-center gap-1.5 text-xs font-medium text-success">
              <Check size={14} />
              {t("versions.installed")}
            </span>
            <Button size="sm" disabled={playDisabled} onClick={onPlay}>
              <Play size={14} />
              {t("versions.play")}
            </Button>
          </div>
        ) : (
          <Button size="sm" onClick={onInstall}>
            <Download size={14} />
            {t("versions.install")}
          </Button>
        )}
      </div>

      {progress && (
        <div className="mt-3">
          <div className="h-1.5 overflow-hidden rounded-full bg-surface-raised">
            <motion.div
              className="h-full rounded-full bg-accent"
              animate={{ width: `${progress.stage === "resolving" ? 3 : percent}%` }}
              transition={{ duration: 0.15 }}
            />
          </div>
          <p className="mt-1.5 text-xs text-muted">
            {progress.stage === "resolving"
              ? t("versions.resolving")
              : t("versions.progress", {
                  done: formatBytes(progress.doneBytes),
                  total: formatBytes(progress.totalBytes),
                  files: progress.doneFiles,
                  totalFiles: progress.totalFiles,
                })}
          </p>
        </div>
      )}

      {error && <p className="mt-2 text-xs text-danger">{error}</p>}
    </div>
  );
}
