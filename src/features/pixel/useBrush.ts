import { useCallback, useEffect, useState } from "react";
import type { PixelDocument } from "./PixelDocument";
import type { BrushSettings, Tool } from "./tools";

const MAX_RECENT = 10;

export const TOOL_KEYS: Record<string, Tool> = { b: "pencil", e: "eraser", g: "fill", i: "picker" };

export function useBrush(doc: PixelDocument) {
  const [tool, setTool] = useState<Tool>("pencil");
  const [color, setColor] = useState("#2f8fd8");
  const [size, setSize] = useState(1);
  const [recent, setRecent] = useState<string[]>([]);

  const chooseColor = useCallback((next: string) => {
    setColor(next);
    setRecent((previous) => [next, ...previous.filter((item) => item !== next)].slice(0, MAX_RECENT));
  }, []);

  const pickColor = useCallback(
    (next: string) => {
      chooseColor(next);
      setTool("pencil");
    },
    [chooseColor],
  );

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement) return;
      const key = event.key.toLowerCase();
      if (event.ctrlKey && key === "z") {
        event.preventDefault();
        if (event.shiftKey) doc.redo();
        else doc.undo();
      } else if (event.ctrlKey && key === "y") {
        event.preventDefault();
        doc.redo();
      } else if (!event.ctrlKey && !event.altKey && TOOL_KEYS[key]) {
        setTool(TOOL_KEYS[key]);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [doc]);

  const brush: BrushSettings = { tool, color, size };
  return { brush, setTool, setSize, chooseColor, pickColor, recent };
}

export type Brush = ReturnType<typeof useBrush>;
