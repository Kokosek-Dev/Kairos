import { useMutation } from "@tanstack/react-query";
import { AnimatePresence, motion } from "framer-motion";
import { Check } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { LanguagePicker } from "@/components/LanguagePicker";
import { Button } from "@/components/ui/button";
import { useActiveAccount } from "@/features/skins/useProfile";
import { useSignInStore } from "@/features/accounts/signInStore";
import { StorageChooser } from "@/features/settings/StorageSection";
import { useVersionManifest } from "@/features/instances/VersionPicker";
import i18n, { getStoredLanguage, setLanguage } from "@/i18n";
import type { LanguageCode } from "@/i18n/locales";
import { queryClient } from "@/lib/queryClient";
import { errorMessage } from "@/lib/tauri/errors";
import { createInstance } from "@/lib/tauri/instances";
import { cn } from "@/lib/utils";

const STORAGE_KEY = "zenith.welcome";
const STEPS = ["language", "storage", "account", "instance"] as const;

/** Players who set up Zenith before this guide existed are not asked again. */
function firstStep(): number {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored !== null) return Number(stored);
  } catch {
    return STEPS.length;
  }
  return getStoredLanguage() === null ? 0 : STEPS.length;
}

function remember(step: number) {
  try {
    localStorage.setItem(STORAGE_KEY, String(step));
  } catch {
    // The guide simply shows again next time when storage is unavailable.
  }
}

function AccountStep() {
  const { t } = useTranslation();
  const { active } = useActiveAccount();
  const start = useSignInStore((state) => state.start);

  return active ? (
    <p className="flex items-center justify-center gap-2 font-medium text-success">
      <Check size={18} />
      {t("signIn.done", { name: active.username })}
    </p>
  ) : (
    <Button variant="secondary" size="lg" className="w-full" onClick={() => void start()}>
      {t("welcome.signIn")}
    </Button>
  );
}

function InstanceStep() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const manifest = useVersionManifest();
  const latest = manifest.data?.latest.release;
  const create = useMutation({
    mutationFn: () =>
      createInstance({
        name: t("instances.defaultName", { version: latest }),
        gameVersion: latest ?? "",
        icon: { kind: "preset", key: "cube" },
        loader: "vanilla",
        loaderVersion: null,
      }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["instances"] });
      navigate("/library");
    },
  });

  if (create.isSuccess) {
    return (
      <p className="flex items-center justify-center gap-2 font-medium text-success">
        <Check size={18} />
        {t("welcome.instanceDone", { name: create.data.name })}
      </p>
    );
  }
  return (
    <>
      <Button size="lg" className="w-full" disabled={!latest || create.isPending} onClick={() => create.mutate()}>
        {t("welcome.instanceCreate")}
      </Button>
      {create.isError && <p className="mt-2 text-sm text-danger">{errorMessage(t, create.error)}</p>}
    </>
  );
}

export function WelcomeDialog() {
  const { t } = useTranslation();
  const [step, setStep] = useState(firstStep);
  const open = step < STEPS.length;
  const last = step === STEPS.length - 1;

  useEffect(() => remember(step), [step]);

  const next = () => {
    if (step === 0) setLanguage(i18n.language as LanguageCode);
    setStep(step + 1);
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
            key={step}
            initial={{ opacity: 0, y: 12, scale: 0.97 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            className="glass w-[min(30rem,calc(100vw-2rem))] rounded-xl p-8 text-center shadow-glow"
          >
            <div className="mb-5 flex justify-center gap-2">
              {STEPS.map((name, index) => (
                <span key={name} className={cn("h-1.5 w-8 rounded-full bg-surface-raised", index <= step && "bg-brand")} />
              ))}
            </div>
            <h1 className="text-2xl font-bold">{t(`welcome.${STEPS[step]}.title`)}</h1>
            <p className="mb-6 mt-2 text-muted">{t(`welcome.${STEPS[step]}.body`)}</p>
            <div className="mb-6 text-left">
              {step === 0 && (
                <div className="flex justify-center">
                  <LanguagePicker />
                </div>
              )}
              {step === 1 && <StorageChooser />}
              {step === 2 && <AccountStep />}
              {step === 3 && <InstanceStep />}
            </div>
            <div className="flex gap-2">
              {step > 0 && (
                <Button variant="ghost" size="lg" className="flex-1" onClick={next}>
                  {t("welcome.skip")}
                </Button>
              )}
              <Button size="lg" className="flex-1" onClick={next}>
                {last ? t("welcome.finish") : t("welcome.continue")}
              </Button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
