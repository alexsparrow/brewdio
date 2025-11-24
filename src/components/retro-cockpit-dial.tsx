import React, { useMemo } from "react";
import { OlFarve } from "@/calculations/olfarve";

// --- Types & Interfaces ---

interface RetroCockpitDialProps {
  /** The text label displayed at the bottom (e.g., "IBU", "SRM") */
  label: string;
  /** The current value to display */
  value: number;
  /** The minimum value of the gauge (default: 0) */
  min?: number;
  /** The maximum value of the gauge (default: 100) */
  max?: number;
  /** The start of the "suggested range" (green arc). Pass null to hide. */
  targetMin?: number | null;
  /** The end of the "suggested range" (green arc). Pass null to hide. */
  targetMax?: number | null;
  /** Width of the component in px or % */
  width?: number | string;
  /** If true, show SRM color gradient in target range */
  showSrmGradient?: boolean;
}

interface CartesianPoint {
  x: number;
  y: number;
}

// --- Helper Functions ---

const polarToCartesian = (
  centerX: number,
  centerY: number,
  radius: number,
  angleInDegrees: number
): CartesianPoint => {
  const angleInRadians = (angleInDegrees - 90) * (Math.PI / 180.0);
  return {
    x: centerX + radius * Math.cos(angleInRadians),
    y: centerY + radius * Math.sin(angleInRadians),
  };
};

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

const describeArc = (
  x: number,
  y: number,
  radius: number,
  startAngle: number,
  endAngle: number
): string => {
  // Prevent artifacts if angles are identical
  if (startAngle === endAngle) return "";

  const start = polarToCartesian(x, y, radius, endAngle);
  const end = polarToCartesian(x, y, radius, startAngle);

  // Ensure consistent arc direction calculation
  const diff = endAngle - startAngle;

  const largeArcFlag = diff <= 180 ? "0" : "1";

  return [
    "M",
    start.x,
    start.y,
    "A",
    radius,
    radius,
    0,
    largeArcFlag,
    0,
    end.x,
    end.y,
  ].join(" ");
};

// --- Component ---

/**
 * Retro-style analog dial gauge component.
 * Features animated needle, tick marks, optional target range, and vintage aesthetic.
 */
export const RetroCockpitDial: React.FC<RetroCockpitDialProps> = ({
  label = "PARAMETER",
  value = 0,
  min = 0,
  max = 100,
  targetMin = null,
  targetMax = null,
  width = 300,
  showSrmGradient = false,
}) => {
  // Gauge Geometry Configuration
  const cx = 150;
  const cy = 150;
  const radius = 120;
  const startAngle = -120;
  const endAngle = 120;
  const totalAngle = endAngle - startAngle;

  // Generate unique IDs for filters to avoid conflicts with multiple dials
  const filterId = React.useMemo(
    () => `dial-${label.toLowerCase().replace(/[^a-z]/g, "")}`,
    [label]
  );

  const arcPoint = (angle: number, radius: number) =>
    polarToCartesian(cx, cy, radius, angle);

  // 1. Calculate Needle Angle
  const clampedValue = Math.min(Math.max(value, min), max);
  const ratio = (clampedValue - min) / (max - min);
  const currentAngle = startAngle + ratio * totalAngle;

  // 2. Determine if value is within target range
  const isInRange = useMemo(() => {
    if (
      targetMin === null ||
      targetMax === null ||
      targetMin === undefined ||
      targetMax === undefined
    ) {
      return true; // Default to in range if no range specified
    }
    // Handle potentially swapped min/max inputs gracefully
    const tMin = Math.min(targetMin, targetMax);
    const tMax = Math.max(targetMin, targetMax);
    return value >= tMin && value <= tMax;
  }, [value, targetMin, targetMax]);

  // 3. Calculate Target Range Arc
  const rangeArcData = useMemo(() => {
    if (
      targetMin === null ||
      targetMax === null ||
      targetMin === undefined ||
      targetMax === undefined
    ) {
      return null;
    }

    // FIX: Sort values to prevent negative arc calculation (the "non-arc shape" bug)
    const rawMin = Math.min(targetMin, targetMax);
    const rawMax = Math.max(targetMin, targetMax);

    // FIX: Clamp range to the dial's min/max limits
    const tMin = Math.max(rawMin, min);
    const tMax = Math.min(rawMax, max);

    // Avoid rendering if range is invalid or completely outside current view
    if (tMin >= tMax || tMin > max || tMax < min) return null;

    const minRatio = (tMin - min) / (max - min);
    const maxRatio = (tMax - min) / (max - min);

    const arcStart = startAngle + minRatio * totalAngle;
    const arcEnd = startAngle + maxRatio * totalAngle;

    return {
      path: describeArc(cx, cy, radius - 10, arcStart, arcEnd),
      startAngle: arcStart,
      endAngle: arcEnd,
      minValue: tMin,
      maxValue: tMax,
    };
  }, [targetMin, targetMax, min, max, startAngle, totalAngle, cx, cy, radius]);

  // 4. Generate Ticks
  const ticks = useMemo(() => {
    const tickCount = 11;
    return Array.from({ length: tickCount }).map((_, i) => {
      const tickRatio = i / (tickCount - 1);
      const angle = startAngle + tickRatio * totalAngle;
      const isMajor = i % 5 === 0;

      const outer = polarToCartesian(cx, cy, radius, angle);
      const inner = polarToCartesian(
        cx,
        cy,
        radius - (isMajor ? 15 : 8),
        angle
      );

      return (
        <line
          key={i}
          x1={outer.x}
          y1={outer.y}
          x2={inner.x}
          y2={inner.y}
          stroke="currentColor"
          className="text-gray-600 dark:text-gray-300"
          strokeWidth={isMajor ? 3 : 1}
          opacity={isMajor ? 1 : 0.6}
        />
      );
    });
  }, [startAngle, totalAngle, cx, cy, radius]);

  return (
    <div
      style={{ width: width, display: "inline-block", position: "relative" }}
    >
      <svg
        viewBox="0 0 300 260"
        style={{ width: "100%", height: "auto", background: "transparent" }}
      >
        <defs>
          {/* Glow filter for needle */}
          <filter
            id={`${filterId}-needleGlow`}
            x="-50%"
            y="-50%"
            width="200%"
            height="200%"
          >
            <feGaussianBlur stdDeviation="3" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>

          {/* Glow filter for target range */}
          <filter
            id={`${filterId}-rangeGlow`}
            x="-20%"
            y="-20%"
            width="140%"
            height="140%"
          >
            <feGaussianBlur stdDeviation="4" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>

          {/* SRM Color Gradient for target range */}
        </defs>

        {showSrmGradient &&
          rangeArcData &&
          (() => {
            const segments = 20;
            const paths = [];
            for (let i = 0; i < segments; i++) {
              const t1 = i / segments;
              const t2 = (i + 1) / segments;

              const angle1 = lerp(
                rangeArcData.startAngle,
                rangeArcData.endAngle,
                t1
              );
              const angle2 = lerp(
                rangeArcData.startAngle,
                rangeArcData.endAngle,
                t2
              );

              const value1 = lerp(
                rangeArcData.minValue,
                rangeArcData.maxValue,
                t1
              );

              const smallPath = describeArc(
                cx,
                cy,
                radius - 10,
                angle1,
                angle2
              );

              paths.push(
                <path
                  key={i}
                  d={smallPath}
                  fill="none"
                  stroke={OlFarve.rgbToHex(OlFarve.srmToSRGB(value1))}
                  strokeWidth="12"
                  strokeLinecap="butt"
                  strokeOpacity="0.9"
                />
              );
            }
            return paths;
          })()}

        {/* Track */}
        <path
          d={describeArc(cx, cy, radius, startAngle, endAngle)}
          fill="none"
          stroke="currentColor"
          className="text-gray-400 dark:text-gray-600"
          strokeWidth="4"
          strokeLinecap="round"
        />

        {/* FIX: Ticks rendered BEFORE Range Arc. 
           This ensures the ticks are conceptually "behind" the glowing range indicator.
        */}
        <g>{ticks}</g>

        {/* Target Range */}
        {rangeArcData && (
          <path
            d={rangeArcData.path}
            fill="none"
            stroke={
              showSrmGradient ? `url(#${filterId}-srmGradient)` : "#10b981"
            }
            strokeWidth="12"
            strokeOpacity={showSrmGradient ? "0.8" : "0.6"}
            strokeLinecap="butt"
            // filter={`url(#${filterId}-rangeGlow)`}
          />
        )}

        {/* Labels */}
        <text
          x={cx}
          y={cy + 60}
          textAnchor="middle"
          className="fill-gray-500 dark:fill-gray-400"
          fontSize="14"
          fontFamily="monospace"
          letterSpacing="2px"
        >
          {label}
        </text>
        <text
          x={cx}
          y={cy + 90}
          textAnchor="middle"
          className={`${isInRange ? "fill-gray-900 dark:fill-white" : "fill-red-600 dark:fill-red-500"} glow-text`}
          fontSize="32"
          fontFamily="monospace"
          fontWeight="bold"
        >
          {label === "ABV" ? `${Math.round(value)}%` : Math.round(value)}
        </text>

        <text
          x={cx - 90}
          y={cy + 80}
          textAnchor="middle"
          className="fill-gray-600 dark:fill-gray-500"
          fontSize="12"
          fontFamily="monospace"
        >
          {min}
        </text>
        <text
          x={cx + 90}
          y={cy + 80}
          textAnchor="middle"
          className="fill-gray-600 dark:fill-gray-500"
          fontSize="12"
          fontFamily="monospace"
        >
          {max}
        </text>

        {/* Needle Group */}
        <g
          style={{
            transition: "transform 0.6s cubic-bezier(0.34, 1.56, 0.64, 1)",
            transformOrigin: `${cx}px ${cy}px`,
            transform: `rotate(${currentAngle}deg)`,
          }}
        >
          {/* Contrasting Shadow for visibility in both modes */}
          <line
            x1={cx}
            y1={cy}
            x2={cx}
            y2={cy - radius + 10}
            className="stroke-white dark:stroke-black"
            strokeWidth="7"
            strokeLinecap="round"
            opacity="0.5"
          />

          {/* Main Needle Body */}
          <line
            x1={cx}
            y1={cy}
            x2={cx}
            y2={cy - radius + 10}
            className="stroke-gray-900 dark:stroke-white"
            strokeWidth="5"
            strokeLinecap="round"
            filter={`url(#${filterId}-needleGlow)`}
          />

          {/* Needle Highlight for 3D effect */}
          <line
            x1={cx}
            y1={cy}
            x2={cx}
            y2={cy - radius + 10}
            className="stroke-gray-600 dark:stroke-gray-300"
            strokeWidth="2"
            strokeLinecap="round"
            style={{ transform: "translateX(-1px)" }}
          />

          {/* Central Hub - Outer Ring */}
          <circle
            cx={cx}
            cy={cy}
            r="10"
            className="fill-gray-900 dark:fill-white"
            opacity="0.2"
          />

          {/* Central Hub - Main Body */}
          <circle
            cx={cx}
            cy={cy}
            r="8"
            className="fill-gray-900 dark:fill-white stroke-gray-900 dark:stroke-white"
            strokeWidth="2"
            filter={`url(#${filterId}-needleGlow)`}
          />

          {/* Central Hub - Highlight */}
          <circle
            cx={cx - 2}
            cy={cy - 2}
            r="3"
            className="fill-gray-600 dark:fill-gray-300"
            opacity="0.6"
          />
        </g>
      </svg>
    </div>
  );
};
