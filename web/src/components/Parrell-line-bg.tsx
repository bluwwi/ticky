import type { CSSProperties, ReactNode } from "react";

type DotBackgroundProps = {
  size?: number;
  dot?: number;
  color?: string;
  bg?: string;
  fade?: boolean;
  className?: string;
  children?: ReactNode;
  style?: CSSProperties;
};

export default function DotBackground({
  size = 20,
  dot = 1.25,
  color = "#131313",
  bg = "#fff",
  fade = false,
  className,
  children,
  style,
}: DotBackgroundProps) {
  return (
    <div
      className={className}
      style={{
        position: "relative",
        backgroundColor: bg,
        backgroundImage: `radial-gradient(${color} ${dot}px, transparent ${dot}px)`,
        backgroundSize: `${size}px ${size}px`,
        ...style,
      }}
    >
      {fade && (
        <div
          aria-hidden="true"
          style={{
            position: "absolute",
            inset: 0,
            pointerEvents: "none",
            background: `radial-gradient(ellipse at center, transparent 80%, ${bg} 100%)`,
          }}
        />
      )}
      <div style={{ position: "relative" }}>{children}</div>
    </div>
  );
}
