import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { errorMessage } from "@/lib/tauri/errors";
import { deleteInstance, type Instance } from "@/lib/tauri/instances";
import { INSTANCES_KEY } from "./useInstances";

interface DeleteInstanceDialogProps {
  instance: Instance | null;
  onClose: () => void;
  onDeleted?: () => void;
}

export function DeleteInstanceDialog({ instance, onClose, onDeleted }: DeleteInstanceDialogProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const remove = useMutation({
    mutationFn: (target: Instance) => deleteInstance(target.id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
      onClose();
      onDeleted?.();
    },
  });

  const close = () => {
    remove.reset();
    onClose();
  };

  return (
    <Dialog open={instance !== null} onOpenChange={(open) => !open && close()}>
      <DialogContent className="w-[min(28rem,calc(100vw-2rem))]">
        <DialogTitle>{t("instances.deleteTitle", { name: instance?.name })}</DialogTitle>
        <DialogDescription>{t("instances.deleteBody")}</DialogDescription>
        {remove.isError && <p className="mt-3 text-sm text-danger">{errorMessage(t, remove.error)}</p>}
        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={close}>
            {t("common.cancel")}
          </Button>
          <Button
            className="bg-danger hover:bg-danger/85"
            disabled={remove.isPending}
            onClick={() => instance && remove.mutate(instance)}
          >
            {t("instances.delete")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
