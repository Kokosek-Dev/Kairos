import { useCallback, useEffect, useState } from "react";
import type { PixelDocument } from "./PixelDocument";
import { SHAPE_TOOLS, type BrushSettings, type Mirror, type Tool } from "./tools";

const MAX_RECENT = 10;
export const MAX_BRUSH = 16;

export const TOOL_KEYS: Record<string, Tool> = { b: "pencil", e: "eraser", g: "fill", i: "picker", l: "line", r: "rect", o: "ellipse" };

/** `advanced` adds the shape tools, which only make sense on a flat picture without a model around it. */
export function useBrush(doc: PixelDocument, advanced = false) {
  const [tool, setTool] = useState<Tool>("pencil");
  const [color, setColor] = useState("#2f8fd8");
  const [size, setSize] = useState(1);
  const [mirror, setMirror] = useState<Mirror>("none");
  const [filled, setFilled] = useState(false);
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
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement || event.target instanceof HTMLTextAreaElement) return;
      const key = event.key.toLowerCase();
      if (event.ctrlKey && key === "z") {
        event.preventDefault();
        if (event.shiftKey) doc.redo();
        else doc.undo();
      } else if (event.ctrlKey && key === "y") {
        event.preventDefault();
        doc.redo();
      } else if (!event.ctrlKey && !event.altKey && TOOL_KEYS[key] && (advanced || !SHAPE_TOOLS.includes(TOOL_KEYS[key]))) {
        setTool(TOOL_KEYS[key]);
      } else if (!event.ctrlKey && !event.altKey && (key === "[" || key === "]")) {
        setSize((current) => Math.min(MAX_BRUSH, Math.max(1, current + (key === "]" ? 1 : -1))));
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [doc, advanced]);

  const brush: BrushSettings = { tool, color, size, mirror, filled };
  return { brush, setTool, setSize, setMirror, setFilled, chooseColor, pickColor, recent };
}

export type Brush = ReturnType<typeof useBrush>;
