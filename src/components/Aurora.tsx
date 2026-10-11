/** Soft colored light drifting behind the whole app. */
export function Aurora() {
  return (
    <div aria-hidden="true" className="pointer-events-none absolute inset-0 overflow-hidden">
      <div
        className="animate-aurora-a absolute -left-1/4 -top-1/3 size-[70vmax] rounded-full opacity-60"
        style={{ background: "radial-gradient(closest-side, oklch(0.5 0.17 var(--accent-hue) / 0.45), transparent)" }}
      />
      <div
        className="animate-aurora-b absolute -bottom-1/3 -right-1/4 size-[65vmax] rounded-full opacity-60"
        style={{ background: "radial-gradient(closest-side, oklch(0.5 0.17 var(--hue-2) / 0.38), transparent)" }}
      />
    </div>
  );
}
