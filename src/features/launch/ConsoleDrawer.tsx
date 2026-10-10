import { AnimatePresence, motion } from "framer-motion";
import { Copy, Eraser, Square, X } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { formatBytes } from "@/lib/format";
import { errorMessage } from "@/lib/tauri/errors";
import { cn } from "@/lib/utils";
import { isBusy, useGameStore, type GamePhase, type LogLine } from "./gameStore";

const BOTTOM_TOLERANCE_PX = 24;

function lineClass(line: LogLine): string {
  if (line.level === "ERROR" || line.level === "FATAL") return "text-danger";
  if (line.level === "WARN") return "text-yellow-400";
  if (line.level === "DEBUG" || line.level === "TRACE") return "text-muted";
  return line.stream === "stderr" ? "text-orange-300" : "text-foreground/85";
}

function useStatusText(phase: GamePhase): string {
  const { t } = useTranslation();
  switch (phase.kind) {
    case "idle":
      return "";
    case "preparing": {
      const label = t(`console.step.${phase.step}`);
      return phase.totalBytes > 0
        ? `${label} · ${formatBytes(phase.doneBytes)} / ${formatBytes(phase.totalBytes)}`
        : label;
    }
    case "running":
      return t("console.running");
    case "exited":
      return t("console.exited", { code: phase.code ?? "?" });
    case "failed":
      return errorMessage(t, phase.error);
  }
}

export function ConsoleDrawer() {
  const { t } = useTranslation();
  const { consoleOpen, phase, lines, title, stop, clear, setConsoleOpen } = useGameStore();
  const status = useStatusText(phase);
  const scroller = useRef<HTMLDivElement>(null);
  const stickToBottom = useRef(true);

  useEffect(() => {
    const element = scroller.current;
    if (element && stickToBottom.current) element.scrollTop = element.scrollHeight;
  }, [lines, consoleOpen]);

  const onScroll = () => {
    const element = scroller.current;
    if (!element) return;
    stickToBottom.current =
      element.scrollHeight - element.scrollTop - element.clientHeight < BOTTOM_TOLERANCE_PX;
  };

  const copy = () => void navigator.clipboard.writeText(lines.map((line) => line.text).join("\n"));

  return (
    <AnimatePresence>
      {consoleOpen && (
        <motion.section
          initial={{ height: 0, opacity: 0 }}
          animate={{ height: 300, opacity: 1 }}
          exit={{ height: 0, opacity: 0 }}
          transition={{ duration: 0.2 }}
          className="glass flex shrink-0 flex-col overflow-hidden border-x-0 border-b-0"
        >
          <header className="flex h-11 shrink-0 items-center gap-3 border-b px-4">
            <span
              className={cn(
                "size-2 rounded-full",
                phase.kind === "running" && "bg-success",
                phase.kind === "preparing" && "animate-pulse bg-accent",
                phase.kind === "failed" && "bg-danger",
                (phase.kind === "exited" || phase.kind === "idle") && "bg-muted",
              )}
            />
            <span className="font-semibold">{title ?? t("console.title")}</span>
            <span className={cn("min-w-0 flex-1 truncate text-xs", phase.kind === "failed" ? "text-danger" : "text-muted")}>
              {status}
            </span>
            {isBusy(phase) && (
              <Button size="sm" variant="secondary" onClick={() => void stop()}>
                <Square size={12} />
                {t("console.stop")}
              </Button>
            )}
            <Button size="sm" variant="ghost" onClick={copy} aria-label={t("console.copy")} title={t("console.copy")}>
              <Copy size={14} />
            </Button>
            <Button size="sm" variant="ghost" onClick={clear} aria-label={t("console.clear")} title={t("console.clear")}>
              <Eraser size={14} />
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setConsoleOpen(false)}
              aria-label={t("console.close")}
              title={t("console.close")}
            >
              <X size={14} />
            </Button>
          </header>
          <div
            ref={scroller}
            onScroll={onScroll}
            className="min-h-0 flex-1 select-text overflow-y-auto bg-background/60 px-4 py-2 font-mono text-xs leading-5"
          >
            {lines.map((line) => (
              <div key={line.id} className={cn("whitespace-pre-wrap break-all", lineClass(line))}>
                {line.text || " "}
              </div>
            ))}
          </div>
        </motion.section>
      )}
    </AnimatePresence>
  );
}
