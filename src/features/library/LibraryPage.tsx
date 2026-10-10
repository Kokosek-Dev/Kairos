import { Boxes, Plus } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { PageHeader } from "@/components/shell/PageHeader";
import { Button } from "@/components/ui/button";
import { DeleteInstanceDialog } from "@/features/instances/DeleteInstanceDialog";
import { InstanceCard } from "@/features/instances/InstanceCard";
import { NewInstanceDialog } from "@/features/instances/NewInstanceDialog";
import { useInstances } from "@/features/instances/useInstances";
import { errorMessage } from "@/lib/tauri/errors";
import type { Instance } from "@/lib/tauri/instances";

export function LibraryPage() {
  const { t } = useTranslation();
  const instances = useInstances();
  const [creating, setCreating] = useState(false);
  const [deleting, setDeleting] = useState<Instance | null>(null);

  return (
    <>
      <div className="flex items-start justify-between gap-4">
        <PageHeader title={t("library.title")} subtitle={t("library.subtitle")} />
        <Button onClick={() => setCreating(true)}>
          <Plus size={16} />
          {t("instances.new")}
        </Button>
      </div>

      {instances.isPending && (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-4">
          {Array.from({ length: 3 }, (_, index) => (
            <div key={index} className="skeleton h-[168px] rounded-xl" />
          ))}
        </div>
      )}

      {instances.isError && (
        <div className="rounded-lg border border-danger/40 bg-danger/10 p-5">
          <p className="mb-3">{errorMessage(t, instances.error)}</p>
          <Button variant="secondary" size="sm" onClick={() => void instances.refetch()}>
            {t("versions.retry")}
          </Button>
        </div>
      )}

      {instances.data?.length === 0 && (
        <div className="grid place-items-center rounded-xl border border-dashed py-20 text-center">
          <Boxes size={40} className="mb-4 text-accent" />
          <h2 className="text-lg font-semibold">{t("library.emptyTitle")}</h2>
          <p className="mb-5 mt-1 text-muted">{t("library.emptyBody")}</p>
          <Button onClick={() => setCreating(true)}>
            <Plus size={16} />
            {t("instances.new")}
          </Button>
        </div>
      )}

      {instances.data && instances.data.length > 0 && (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-4">
          {instances.data.map((instance) => (
            <InstanceCard key={instance.id} instance={instance} onDelete={setDeleting} />
          ))}
        </div>
      )}

      <NewInstanceDialog open={creating} onOpenChange={setCreating} />
      <DeleteInstanceDialog instance={deleting} onClose={() => setDeleting(null)} />
    </>
  );
}
