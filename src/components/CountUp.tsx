import { animate, useMotionValue } from "framer-motion";
import { useEffect, useRef } from "react";

interface CountUpProps {
  value: number;
  format: (value: number) => string;
}

/** A number that counts up to its value when it first appears. */
export function CountUp({ value, format }: CountUpProps) {
  const element = useRef<HTMLSpanElement>(null);
  const shown = useMotionValue(0);

  useEffect(() => {
    const unsubscribe = shown.on("change", (current) => {
      if (element.current) element.current.textContent = format(Math.round(current));
    });
    const controls = animate(shown, value, { duration: 1.1, ease: "easeOut" });
    return () => {
      controls.stop();
      unsubscribe();
    };
  }, [shown, value, format]);

  return <span ref={element}>{format(0)}</span>;
}
