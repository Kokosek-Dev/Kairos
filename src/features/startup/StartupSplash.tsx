import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";

const DURATION_MS = 2600;
const WORD = "KAIROS".split("");
const STROKES = ["M20 14V50", "M25 33L44 50", "M25 31L39 18"];

/** The animation that plays while Kairos opens: the mark draws itself, then the name settles in. */
export function StartupSplash() {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    const timer = window.setTimeout(() => setVisible(false), DURATION_MS);
    return () => window.clearTimeout(timer);
  }, []);

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          key="startup"
          exit={{ opacity: 0, scale: 1.06 }}
          transition={{ duration: 0.6, ease: "easeInOut" }}
          className="fixed inset-x-0 bottom-0 top-11 z-[70] grid place-items-center overflow-hidden bg-background"
        >
          <div className="grid-pattern pointer-events-none absolute inset-0" />
          <motion.div
            className="pointer-events-none absolute size-96 rounded-full bg-accent/25 blur-3xl"
            initial={{ opacity: 0, scale: 0.6 }}
            animate={{ opacity: [0, 1, 0.7], scale: [0.6, 1.1, 1] }}
            transition={{ duration: 1.8, ease: "easeOut" }}
          />

          <div className="relative flex flex-col items-center">
            <svg viewBox="0 0 64 64" className="size-36 drop-shadow-[0_0_30px_var(--accent)]" aria-hidden="true">
              <defs>
                <linearGradient id="kairos-startup" x1="14" y1="10" x2="50" y2="54" gradientUnits="userSpaceOnUse">
                  <stop style={{ stopColor: "var(--accent)" }} />
                  <stop offset="1" style={{ stopColor: "var(--accent-2)" }} />
                </linearGradient>
              </defs>
              <g fill="none" stroke="url(#kairos-startup)" strokeWidth="9" strokeLinecap="round" strokeLinejoin="round">
                {STROKES.map((path, index) => (
                  <motion.path
                    key={path}
                    d={path}
                    initial={{ pathLength: 0, opacity: 0 }}
                    animate={{ pathLength: 1, opacity: 1 }}
                    transition={{
                      pathLength: { duration: 0.55, delay: 0.2 + index * 0.35, ease: "easeInOut" },
                      opacity: { duration: 0.01, delay: 0.2 + index * 0.35 },
                    }}
                  />
                ))}
              </g>
              <motion.path
                d="M47 3l2.4 6.6L56 12l-6.6 2.4L47 21l-2.4-6.6L38 12l6.6-2.4z"
                style={{ fill: "var(--foreground)", transformBox: "fill-box", transformOrigin: "center" }}
                initial={{ scale: 0, rotate: -120, opacity: 0 }}
                animate={{ scale: [0, 1.5, 1], rotate: 0, opacity: 1 }}
                transition={{ duration: 0.7, delay: 1.25, ease: "easeOut" }}
              />
            </svg>

            <div className="mt-6 flex gap-3 text-3xl font-extrabold">
              {WORD.map((letter, index) => (
                <motion.span
                  key={`${letter}${index}`}
                  className="text-gradient"
                  initial={{ opacity: 0, y: 16, filter: "blur(6px)" }}
                  animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
                  transition={{ duration: 0.45, delay: 1.1 + index * 0.07, ease: "easeOut" }}
                >
                  {letter}
                </motion.span>
              ))}
            </div>

            <div className="mt-8 h-1 w-48 overflow-hidden rounded-full bg-surface-raised">
              <motion.div
                className="h-full rounded-full bg-brand"
                initial={{ width: "0%" }}
                animate={{ width: "100%" }}
                transition={{ duration: (DURATION_MS - 500) / 1000, ease: "easeInOut" }}
              />
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
