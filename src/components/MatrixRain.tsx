import { useEffect, useRef } from "react";

const GLYPHS = "01アイウエオカキクケコサシスセソ{}<>/=+*#$%";
const FONT_SIZE = 15;
const FRAME_MS = 55;

/** Falling green characters behind everything while incognito is on. */
export function MatrixRain() {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const element = canvas.current;
    const context = element?.getContext("2d");
    if (!element || !context || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    let drops: number[] = [];
    const resize = () => {
      element.width = element.clientWidth;
      element.height = element.clientHeight;
      drops = Array.from({ length: Math.ceil(element.width / FONT_SIZE) }, () => Math.random() * -50);
    };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(element);

    let timer = 0;
    const draw = () => {
      context.fillStyle = "rgba(0, 0, 0, 0.12)";
      context.fillRect(0, 0, element.width, element.height);
      context.font = `${FONT_SIZE}px monospace`;
      drops.forEach((row, column) => {
        const glyph = GLYPHS[Math.floor(Math.random() * GLYPHS.length)];
        context.fillStyle = row % 7 < 1 ? "#d8ffe6" : "#12c15a";
        context.fillText(glyph, column * FONT_SIZE, row * FONT_SIZE);
        drops[column] = row * FONT_SIZE > element.height && Math.random() > 0.975 ? 0 : row + 1;
      });
      timer = window.setTimeout(draw, FRAME_MS);
    };
    draw();

    return () => {
      window.clearTimeout(timer);
      observer.disconnect();
    };
  }, []);

  return <canvas ref={canvas} className="absolute inset-0 size-full opacity-30" />;
}
