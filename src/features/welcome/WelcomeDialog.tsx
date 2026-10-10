import { AnimatePresence, motion } from "framer-motion";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { LanguagePicker } from "@/components/LanguagePicker";
import i18n, { getStoredLanguage, setLanguage } from "@/i18n";
import type { LanguageCode } from "@/i18n/locales";

export function WelcomeDialog() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(() => getStoredLanguage() === null);

  const confirm = () => {
    setLanguage(i18n.language as LanguageCode);
    setOpen(false);
  };

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-50 grid place-items-center bg-background/70 backdrop-blur-md"
        >
          <motion.div
            initial={{ opacity: 0, y: 12, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            className="glass w-[26rem] rounded-xl p-8 text-center shadow-glow"
          >
            <h1 className="text-2xl font-bold">{t("welcome.title")}</h1>
            <p className="mb-6 mt-2 text-muted">{t("welcome.body")}</p>
            <div className="mb-6 flex justify-center">
              <LanguagePicker />
            </div>
            <Button size="lg" className="w-full" onClick={confirm}>
              {t("welcome.continue")}
            </Button>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
