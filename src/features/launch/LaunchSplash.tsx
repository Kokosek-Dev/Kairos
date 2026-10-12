import { AnimatePresence, motion } from "framer-motion";
import { Check } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Logo } from "@/components/Logo";
import { Button } from "@/components/ui/button";
import { formatBytes } from "@/lib/format";
import { useGameStore } from "./gameStore";

/** How long the splash stays after the game window opens, so the hand-over does not feel abrupt. */
const LINGER_MS = 1600;
const RINGS = [0, 1, 2];

/** A full-window start screen shown while the game is being prepared and started. */
export function LaunchSplash() {
  const { t } = useTranslation();
  const { phase, title, startedAt, stop } = useGameStore();
  const [lingerFor, setLingerFor] = useState<number | null>(null);
  const [hiddenFor, setHiddenFor] = useState<number | null>(null);

  useEffect(() => {
    if (phase.kind !== "running" || startedAt === null) return;
    setLingerFor(startedAt);
    const timer = window.setTimeout(() => setLingerFor(null), LINGER_MS);
    return () => window.clearTimeout(timer);
  }, [phase.kind, startedAt]);

  const preparing = phase.kind === "preparing";
  const started = phase.kind === "running" && lingerFor === startedAt;
  const visible = (preparing || started) && hiddenFor !== startedAt;
  const progress = phase.kind === "preparing" && phase.totalBytes > 0 ? phase.doneBytes / phase.totalBytes : null;

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          key="launch-splash"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0, scale: 1.03 }}
          transition={{ duration: 0.35, ease: "easeOut" }}
          className="fixed inset-x-0 bottom-0 top-11 z-30 grid place-items-center bg-background/90 backdrop-blur-xl"
        >
          <div className="grid-pattern pointer-events-none absolute inset-0" />
          <div className="relative flex w-[min(26rem,calc(100vw-3rem))] flex-col items-center text-center">
            <div className="relative grid size-40 place-items-center">
              {RINGS.map((ring) => (
                <motion.span
                  key={ring}
                  className="absolute size-24 rounded-full border border-accent/50"
                  initial={{ scale: 0.8, opacity: 0.7 }}
                  animate={started ? { scale: 2.2, opacity: 0 } : { scale: [0.8, 1.9], opacity: [0.7, 0] }}
                  transition={{ duration: started ? 0.8 : 2.4, repeat: started ? 0 : Infinity, delay: started ? 0 : ring * 0.8, ease: "easeOut" }}
                />
              ))}
              <motion.div
                animate={started ? { scale: [1, 1.18, 1] } : { y: [0, -6, 0] }}
                transition={started ? { duration: 0.6 } : { duration: 2.6, repeat: Infinity, ease: "easeInOut" }}
                className="drop-shadow-[0_0_28px_var(--accent)]"
              >
                <Logo className="size-24" />
              </motion.div>
            </div>

            <h2 className="text-gradient mt-2 max-w-full truncate text-3xl font-extrabold tracking-tight">{title}</h2>

            {started ? (
              <p className="mt-4 flex items-center gap-2 font-medium text-success">
                <Check size={18} />
                {t("launch.started")}
              </p>
            ) : (
              <>
                <p className="mt-4 text-muted">{phase.kind === "preparing" ? t(`console.step.${phase.step}`) : ""}</p>
                <div className="mt-4 h-2 w-full overflow-hidden rounded-full bg-surface-raised">
                  {progress === null ? (
                    <motion.div
                      className="h-full w-1/3 rounded-full bg-brand"
                      animate={{ x: ["-100%", "300%"] }}
                      transition={{ duration: 1.4, repeat: Infinity, ease: "easeInOut" }}
                    />
                  ) : (
                    <div className="h-full rounded-full bg-brand transition-all" style={{ width: `${progress * 100}%` }} />
                  )}
                </div>
                <p className="mt-2 h-4 text-xs text-muted">
                  {phase.kind === "preparing" && phase.totalBytes > 0 ? `${formatBytes(phase.doneBytes)} / ${formatBytes(phase.totalBytes)}` : ""}
                </p>
                <div className="mt-6 flex gap-2">
                  <Button variant="ghost" onClick={() => setHiddenFor(startedAt)}>
                    {t("launch.hide")}
                  </Button>
                  <Button variant="secondary" onClick={() => void stop()}>
                    {t("common.cancel")}
                  </Button>
                </div>
              </>
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
