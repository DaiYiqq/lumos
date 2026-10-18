import { useId } from "react";

import { cn } from "@/lib/utils";

interface LogoProps {
  /** 渲染尺寸（像素），默认 28。 */
  size?: number;
  /** 附加样式类。 */
  className?: string;
}

/**
 * Lumos 品牌标记：深色圆角底 + 打开的书 + 一束向上的光。
 *
 * 与安装包图标同源，保证窗口内外的品牌一致。
 */
export function Logo({ size = 28, className }: LogoProps) {
  const gradientId = useId();
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      role="img"
      aria-label="Lumos"
      className={cn("shrink-0", className)}
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#1b2140" />
          <stop offset="100%" stopColor="#0b0e1c" />
        </linearGradient>
        <linearGradient id={`${gradientId}-beam`} x1="0" y1="1" x2="0" y2="0">
          <stop offset="0%" stopColor="#33d6c3" />
          <stop offset="100%" stopColor="#a79bff" />
        </linearGradient>
      </defs>

      {/* 底座 */}
      <rect x="1" y="1" width="30" height="30" rx="9" fill={`url(#${gradientId})`} />
      <rect
        x="1"
        y="1"
        width="30"
        height="30"
        rx="9"
        fill="none"
        stroke="#2c3563"
        strokeWidth="1"
      />

      {/* 向上的一束光 */}
      <path
        d="M16 6.4 19.1 13.2 16 12.1 12.9 13.2Z"
        fill={`url(#${gradientId}-beam)`}
        opacity="0.95"
      />
      <path d="M16 14.2 17.2 18.4 16 17.7 14.8 18.4Z" fill="#cfc8ff" opacity="0.7" />

      {/* 打开的书 */}
      <path
        d="M7.2 19.4 15.2 21.2 15.2 26.1 7.2 24.2Z"
        fill="#f4f1ff"
        stroke="#cfc8f2"
        strokeWidth="0.7"
        strokeLinejoin="round"
      />
      <path
        d="M24.8 19.4 16.8 21.2 16.8 26.1 24.8 24.2Z"
        fill="#e6e1ff"
        stroke="#cfc8f2"
        strokeWidth="0.7"
        strokeLinejoin="round"
      />
      <path d="M16 21.2 16 26.1" stroke="#b9b0ea" strokeWidth="1" strokeLinecap="round" />
    </svg>
  );
}
