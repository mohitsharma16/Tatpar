// ============================================================
// Tatpar — Logo
// Monogram "T" mark: bold white glyph on an accent-blue squircle badge
// ============================================================

interface LogoProps {
  size?: number;
}

export function Logo({ size = 22 }: LogoProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 100 100"
      role="img"
      aria-label="Tatpar"
    >
      <rect width="100" height="100" rx="22" fill="#007acc" />
      <rect x="24" y="25" width="52" height="14" rx="7" fill="#ffffff" />
      <rect x="43" y="25" width="14" height="50" rx="7" fill="#ffffff" />
    </svg>
  );
}
