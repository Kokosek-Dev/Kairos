import { motion } from "framer-motion";
import type { ReactNode } from "react";

const STAGGER_SECONDS = 0.06;
const MAX_STAGGERED = 10;

/** Fades and lifts its content into place, one after another when several appear together. */
export function Reveal({ index = 0, className, children }: { index?: number; className?: string; children: ReactNode }) {
  return (
    <motion.div
      className={className}
      initial={{ opacity: 0, y: 14 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35, ease: "easeOut", delay: Math.min(index, MAX_STAGGERED) * STAGGER_SECONDS }}
    >
      {children}
    </motion.div>
  );
}
