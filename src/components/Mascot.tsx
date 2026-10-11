import { motion } from "framer-motion";
import { create } from "zustand";
import { useEffect, useRef, useState } from "react";
import { RunningAnimation, SkinViewer, WaveAnimation } from "skinview3d";
import { usePreferences } from "@/features/settings/preferences";
import { SkinDocument } from "@/features/skins/editor/SkinDocument";
import { useProfile } from "@/features/skins/useProfile";

export type Act = "run" | "peek";

interface MascotRequests {
  pending: Act | null;
  show: (act: Act) => void;
}

/** Lets the settings page ask for the mascot right away instead of waiting for its next visit. */
export const useMascotRequests = create<MascotRequests>((set) => ({ pending: null, show: (act) => set({ pending: act }) }));

const WIDTH = 130;
const HEIGHT = 170;
const FIRST_DELAY = [20_000, 40_000];
const NEXT_DELAY = [90_000, 240_000];

function between([low, high]: number[]): number {
  return low + Math.random() * (high - low);
}

/** The player's own skin, or a plain figure when nobody is signed in. */
function Figure({ act }: { act: Act }) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const profile = useProfile();
  const skinUrl = profile.data?.skinUrl ?? null;
  const variant = profile.data?.variant === "slim" ? "slim" : "default";

  useEffect(() => {
    const element = canvas.current;
    if (!element) return;
    const viewer = new SkinViewer({ canvas: element, width: WIDTH, height: HEIGHT, zoom: 0.8 });
    viewer.controls.enabled = false;
    viewer.animation = act === "run" ? new RunningAnimation() : new WaveAnimation();
    viewer.playerWrapper.rotation.y = act === "run" ? Math.PI / 2 : -0.4;
    const fallback = () => viewer.loadSkin(new SkinDocument().canvas);
    if (skinUrl) void viewer.loadSkin(skinUrl, { model: variant }).catch(fallback);
    else fallback();
    return () => viewer.dispose();
  }, [act, skinUrl, variant]);

  return <canvas ref={canvas} />;
}

/** Now and then a little player runs across the window or pops up in the corner to wave. */
export function Mascot() {
  const enabled = usePreferences((state) => state.mascot);
  const [act, setAct] = useState<Act | null>(null);
  const appeared = useRef(false);
  const pending = useMascotRequests((state) => state.pending);

  useEffect(() => {
    if (pending === null || act !== null) return;
    useMascotRequests.setState({ pending: null });
    setAct(pending);
  }, [pending, act]);

  useEffect(() => {
    if (!enabled || act !== null || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const timer = setTimeout(() => {
      appeared.current = true;
      setAct(Math.random() < 0.5 ? "run" : "peek");
    }, between(appeared.current ? NEXT_DELAY : FIRST_DELAY));
    return () => clearTimeout(timer);
  }, [enabled, act]);

  const done = () => setAct(null);
  const width = window.innerWidth;

  return (
    <div aria-hidden="true" className="pointer-events-none fixed bottom-0 left-0 z-30">
      {act === "run" && (
        <motion.div
          initial={{ x: -WIDTH }}
          animate={{ x: width }}
          transition={{ duration: Math.max(6, width / 170), ease: "linear" }}
          onAnimationComplete={done}
        >
          <Figure act="run" />
        </motion.div>
      )}
      {act === "peek" && (
        <motion.div
          initial={{ x: width - WIDTH - 32, y: HEIGHT }}
          animate={{ y: [HEIGHT, 6, 6, HEIGHT] }}
          transition={{ duration: 5, times: [0, 0.15, 0.85, 1], ease: "easeInOut" }}
          onAnimationComplete={done}
        >
          <Figure act="peek" />
        </motion.div>
      )}
    </div>
  );
}
