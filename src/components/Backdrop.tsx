import { MatrixRain } from "@/components/MatrixRain";
import { useIncognito } from "@/features/incognito/useIncognito";
import { usePreferences } from "@/features/settings/preferences";

function Glow({ className, hue, strength }: { className: string; hue: string; strength: number }) {
  return (
    <div
      className={`absolute rounded-full opacity-60 ${className}`}
      style={{ background: `radial-gradient(closest-side, oklch(0.5 0.17 ${hue} / ${strength}), transparent)` }}
    />
  );
}

/** What sits behind the whole app: drifting light, a still glow, nothing, or the player's own picture. */
export function Backdrop() {
  const { background, backgroundImage, backgroundDim } = usePreferences();
  const incognito = useIncognito();

  if (incognito) {
    return (
      <div aria-hidden="true" className="pointer-events-none absolute inset-0 overflow-hidden bg-black">
        <MatrixRain />
      </div>
    );
  }

  return (
    <div aria-hidden="true" className="pointer-events-none absolute inset-0 overflow-hidden">
      {background === "aurora" && (
        <>
          <Glow className="animate-aurora-a -left-1/4 -top-1/3 size-[70vmax]" hue="var(--accent-hue)" strength={0.45} />
          <Glow className="animate-aurora-b -bottom-1/3 -right-1/4 size-[65vmax]" hue="var(--hue-2)" strength={0.38} />
        </>
      )}
      {background === "calm" && (
        <>
          <Glow className="-left-1/4 -top-1/3 size-[70vmax]" hue="var(--accent-hue)" strength={0.4} />
          <Glow className="-bottom-1/3 -right-1/4 size-[65vmax]" hue="var(--hue-2)" strength={0.32} />
        </>
      )}
      {background === "plain" && <div className="absolute inset-0 bg-background" />}
      {background === "image" && backgroundImage && (
        <>
          <img src={backgroundImage} alt="" className="absolute inset-0 size-full object-cover" draggable={false} />
          <div className="absolute inset-0 bg-background" style={{ opacity: backgroundDim }} />
        </>
      )}
    </div>
  );
}
