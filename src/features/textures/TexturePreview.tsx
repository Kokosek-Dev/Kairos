import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Segmented } from "@/components/ui/segmented";
import type { PixelDocument } from "@/features/pixel/PixelDocument";
import { getTextureModel } from "@/lib/tauri/texturepacks";
import { cn } from "@/lib/utils";
import { HumanoidPreview, isHumanoid } from "./HumanoidPreview";
import { TexturePreview3D } from "./TexturePreview3D";
import { TilePreview } from "./TilePreview";

type Mode = "3d" | "tiles";

interface TexturePreviewProps {
  packId: string;
  path: string;
  doc: PixelDocument;
  className?: string;
}

/** The texture on the shape the game draws it on, or repeated as tiles for a block. */
export function TexturePreview({ packId, path, doc, className }: TexturePreviewProps) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<Mode>("3d");
  const isBlock = path.startsWith("block/");
  const model = useQuery({ queryKey: ["texture-model", packId, path], queryFn: () => getTextureModel(packId, path), enabled: isBlock, staleTime: Infinity });
  const humanoid = isHumanoid(path, doc);
  const tiles = isBlock && mode === "tiles";

  let kind = "flat";
  if (humanoid) kind = "mob";
  else if (path.startsWith("item/")) kind = "item";
  else if (isBlock) kind = model.data ? "block" : "cube";

  return (
    <div className={cn("panel relative overflow-hidden", className)}>
      {tiles ? (
        <TilePreview doc={doc} />
      ) : humanoid ? (
        <HumanoidPreview path={path} doc={doc} />
      ) : (
        <TexturePreview3D packId={packId} path={path} doc={doc} model={isBlock ? (model.isPending ? undefined : (model.data ?? null)) : null} />
      )}
      <p className="pointer-events-none absolute left-3 top-2 text-xs font-medium text-muted">{t(tiles ? "textures.shape.tiles" : `textures.shape.${kind}`)}</p>
      {isBlock && (
        <div className="absolute right-2 top-1.5 w-36">
          <Segmented
            value={mode}
            onChange={setMode}
            options={[
              { value: "3d", label: t("textures.view3d") },
              { value: "tiles", label: t("textures.viewTiles") },
            ]}
          />
        </div>
      )}
      {!tiles && <p className="pointer-events-none absolute inset-x-0 bottom-2 text-center text-xs text-muted">{t("textures.previewHint")}</p>}
    </div>
  );
}
