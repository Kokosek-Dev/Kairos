import { useEffect } from "react";

const SELECTOR = ".panel, .glow-follow";

/** Lets a soft light follow the pointer across cards. The look itself lives in the stylesheet. */
export function GlowFollow() {
  useEffect(() => {
    const card = (target: EventTarget | null) => (target instanceof Element ? target.closest<HTMLElement>(SELECTOR) : null);

    const move = (event: PointerEvent) => {
      const element = card(event.target);
      if (!element) return;
      const box = element.getBoundingClientRect();
      element.style.setProperty("--mx", `${event.clientX - box.left}px`);
      element.style.setProperty("--my", `${event.clientY - box.top}px`);
      element.style.setProperty("--glow", "1");
    };
    const leave = (event: PointerEvent) => {
      const element = card(event.target);
      if (element && !element.contains(event.relatedTarget as Node | null)) element.style.setProperty("--glow", "0");
    };

    document.addEventListener("pointermove", move);
    document.addEventListener("pointerout", leave);
    return () => {
      document.removeEventListener("pointermove", move);
      document.removeEventListener("pointerout", leave);
    };
  }, []);

  return null;
}
