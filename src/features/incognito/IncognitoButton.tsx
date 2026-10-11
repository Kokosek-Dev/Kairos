import { AnimatePresence, motion } from "framer-motion";
import { VenetianMask } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Logo } from "@/components/Logo";
import { usePreferences } from "@/features/settings/preferences";
import { cn } from "@/lib/utils";
import { useCanUseIncognito, useIncognito } from "./useIncognito";

/** The K of the title bar. It turns into a mask under the pointer, and a click goes in or out of incognito. */
export function IncognitoButton() {
  const { t } = useTranslation();
  const incognito = useIncognito();
  const canUse = useCanUseIncognito();
  const update = usePreferences((state) => state.update);
  const [hovered, setHovered] = useState(false);
  const showMask = hovered || incognito;

  const toggle = () => {
    if (!canUse) {
      toast.info(t("incognito.needsAccount"));
      return;
    }
    update({ incognito: !incognito });
  };

  return (
    <button
      type="button"
      aria-pressed={incognito}
      aria-label={t("incognito.title")}
      title={incognito ? t("incognito.leave") : canUse ? t("incognito.title") : t("incognito.needsAccount")}
      onClick={toggle}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
      className="relative grid size-8 place-items-center rounded-lg transition-colors hover:bg-accent/10"
    >
      {incognito && <span className="absolute inset-0 animate-ping rounded-lg bg-accent/20 [animation-iteration-count:2]" />}
      <AnimatePresence mode="wait" initial={false}>
        <motion.span
          key={showMask ? "mask" : "logo"}
          initial={{ rotateY: 90, scale: 0.6, opacity: 0 }}
          animate={{ rotateY: 0, scale: 1, opacity: 1 }}
          exit={{ rotateY: -90, scale: 0.6, opacity: 0 }}
          transition={{ duration: 0.18, ease: "easeOut" }}
          className="grid place-items-center"
        >
          {showMask ? (
            <VenetianMask size={20} className={cn("text-accent", incognito && "drop-shadow-[0_0_8px_var(--accent)]")} />
          ) : (
            <Logo className="size-5" />
          )}
        </motion.span>
      </AnimatePresence>
    </button>
  );
}
