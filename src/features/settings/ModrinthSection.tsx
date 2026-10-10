import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Field, Input } from "@/components/ui/input";
import { errorMessage } from "@/lib/tauri/errors";
import { getModrinthContact, setModrinthContact } from "@/lib/tauri/modrinth";

const CONTACT_KEY = ["modrinth-contact"];

export function ModrinthSection() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const saved = useQuery({ queryKey: CONTACT_KEY, queryFn: getModrinthContact });
  const [value, setValue] = useState("");

  useEffect(() => {
    if (saved.data !== undefined) setValue(saved.data ?? "");
  }, [saved.data]);

  const save = useMutation({
    mutationFn: () => setModrinthContact(value.trim() || null),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: CONTACT_KEY }),
  });

  return (
    <section className="rounded-xl border bg-surface p-5">
      <h2 className="font-semibold">{t("settings.modrinth")}</h2>
      <p className="mb-4 mt-1 text-muted">{t("settings.modrinthHint")}</p>
      <Field label={t("settings.contact")} hint={t("settings.contactHint")}>
        <div className="flex gap-2">
          <Input value={value} onChange={(event) => setValue(event.target.value)} placeholder="you@example.com" maxLength={100} />
          <Button variant="secondary" disabled={save.isPending} onClick={() => save.mutate()}>
            {t("instances.save")}
          </Button>
        </div>
      </Field>
      {save.isError && <p className="mt-2 text-sm text-danger">{errorMessage(t, save.error)}</p>}
    </section>
  );
}
