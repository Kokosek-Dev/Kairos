import { useMutation } from "@tanstack/react-query";
import { AnimatePresence, motion } from "framer-motion";
import { ArrowLeft, Check, HardDrive, Languages, Play, UserRound, type LucideIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { LanguagePicker } from "@/components/LanguagePicker";
import { Logo } from "@/components/Logo";
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

const STORAGE_KEY = "kairos.welcome.steps";
const EARLIER_KEY = "kairos.welcome";
const STEPS = ["language", "storage", "account", "instance"] as const;
const ICONS: Record<(typeof STEPS)[number], LucideIcon> = {
  language: Languages,
  storage: HardDrive,
  account: UserRound,
  instance: Play,
};

/** Players who set up Kairos before this guide existed, are not asked again. */
function firstStep(): number {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored !== null) return Number(stored);
    const earlier = localStorage.getItem(EARLIER_KEY);
    if (earlier !== null) return Number(earlier) >= 4 ? STEPS.length : 0;
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

function Progress({ step }: { step: number }) {
  return (
    <div className="mb-6 flex items-center justify-center">
      {STEPS.map((name, index) => {
        const Icon = ICONS[name];
        const done = index < step;
        return (
          <div key={name} className="flex items-center">
            <motion.span
              animate={{ scale: index === step ? 1.12 : 1 }}
              className={cn(
                "grid size-9 place-items-center rounded-full border transition-colors",
                done && "border-transparent bg-brand text-accent-foreground",
                index === step && "border-accent bg-accent/15 text-accent shadow-glow",
                index > step && "bg-surface-raised/60 text-muted",
              )}
            >
              {done ? <Check size={16} /> : <Icon size={16} />}
            </motion.span>
            {index < STEPS.length - 1 && <span className={cn("h-0.5 w-6 transition-colors", index < step ? "bg-brand" : "bg-surface-raised")} />}
          </div>
        );
      })}
    </div>
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
          exit={{ opacity: 0, scale: 1.04 }}
          transition={{ duration: 0.35 }}
          className="fixed inset-0 z-50 grid place-items-center overflow-hidden bg-background/80 backdrop-blur-xl"
        >
          <div className="grid-pattern pointer-events-none absolute inset-0" />
          <div className="animate-drift pointer-events-none absolute -left-24 -top-24 size-96 rounded-full bg-accent/25 blur-3xl" />
          <div className="animate-drift pointer-events-none absolute -bottom-32 -right-20 size-96 rounded-full bg-accent-strong/20 blur-3xl [animation-delay:-7s]" />

          <motion.div
            initial={{ opacity: 0, y: 24, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            transition={{ type: "spring", stiffness: 260, damping: 26 }}
            className="glass relative w-[min(36rem,calc(100vw-2rem))] rounded-2xl p-8 text-center shadow-glow"
          >
            <Logo className="mx-auto mb-4 size-14 drop-shadow-[0_0_18px_var(--accent)]" />
            <Progress step={step} />
            <AnimatePresence mode="wait">
              <motion.div
                key={step}
                initial={{ opacity: 0, x: 28 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: -28 }}
                transition={{ duration: 0.22, ease: "easeOut" }}
              >
                <h1 className="text-gradient text-2xl font-extrabold tracking-tight">{t(`welcome.${STEPS[step]}.title`)}</h1>
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
              </motion.div>
            </AnimatePresence>
            <div className="flex gap-2">
              {step > 0 && (
                <Button variant="ghost" size="lg" onClick={() => setStep(step - 1)}>
                  <ArrowLeft size={16} />
                  {t("welcome.back")}
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
