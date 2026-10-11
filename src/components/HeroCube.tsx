import { cn } from "@/lib/utils";

const FACES = [
  "translateZ(var(--half))",
  "rotateY(180deg) translateZ(var(--half))",
  "rotateY(90deg) translateZ(var(--half))",
  "rotateY(-90deg) translateZ(var(--half))",
  "rotateX(90deg) translateZ(var(--half))",
  "rotateX(-90deg) translateZ(var(--half))",
];

/** A slowly turning cube in the launcher colors. Pure CSS, no WebGL. */
export function HeroCube({ size = 120, className }: { size?: number; className?: string }) {
  return (
    <div
      aria-hidden="true"
      className={cn("[perspective:900px]", className)}
      style={{ width: size, height: size, ["--half" as string]: `${size / 2}px` }}
    >
      <div className="relative size-full [transform-style:preserve-3d] [animation:cube-bob_6s_ease-in-out_infinite]">
        <div className="relative size-full [transform-style:preserve-3d] [animation:cube-spin_18s_linear_infinite]">
          {FACES.map((transform, index) => (
            <div
              key={transform}
              className="absolute inset-0 rounded-lg border border-white/25"
              style={{
                transform,
                background: `linear-gradient(135deg, oklch(${0.72 - index * 0.03} 0.17 calc(var(--accent-hue) + ${index * 12}) / 0.85), oklch(0.5 0.2 calc(var(--hue-2) + ${index * 8}) / 0.75))`,
                boxShadow: "inset 0 0 24px oklch(1 0 0 / 0.18)",
              }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
