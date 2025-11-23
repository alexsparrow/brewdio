import React, { useMemo } from 'react';

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
  
  const largeArcFlag = diff <= 180 ? '0' : '1';

  return [
    'M', start.x, start.y,
    'A', radius, radius, 0, largeArcFlag, 0, end.x, end.y
  ].join(' ');
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
  showSrmGradient = false
}) => {
  // Gauge Geometry Configuration
  const cx = 150;
  const cy = 150;
  const radius = 120;
  const startAngle = -120;
  const endAngle = 120;
  const totalAngle = endAngle - startAngle;

  // Generate unique IDs for filters to avoid conflicts with multiple dials
  const filterId = React.useMemo(() => `dial-${label.toLowerCase().replace(/[^a-z]/g, '')}`, [label]);

  // 1. Calculate Needle Angle
  const clampedValue = Math.min(Math.max(value, min), max);
  const ratio = (clampedValue - min) / (max - min);
  const currentAngle = startAngle + (ratio * totalAngle);

  // 2. Determine if value is within target range
  const isInRange = useMemo(() => {
    if (targetMin === null || targetMax === null || targetMin === undefined || targetMax === undefined) {
      return true; // Default to in range if no range specified
    }
    // Handle potentially swapped min/max inputs gracefully
    const tMin = Math.min(targetMin, targetMax);
    const tMax = Math.max(targetMin, targetMax);
    return value >= tMin && value <= tMax;
  }, [value, targetMin, targetMax]);

  // 3. Calculate Target Range Arc
  const rangeArcData = useMemo(() => {
    if (targetMin === null || targetMax === null || targetMin === undefined || targetMax === undefined) {
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

    const arcStart = startAngle + (minRatio * totalAngle);
    const arcEnd = startAngle + (maxRatio * totalAngle);

    return {
      path: describeArc(cx, cy, radius - 10, arcStart, arcEnd),
      startAngle: arcStart,
      endAngle: arcEnd,
      minValue: tMin,
      maxValue: tMax
    };
  }, [targetMin, targetMax, min, max, startAngle, totalAngle, cx, cy, radius]);

  // 4. Generate Ticks
  const ticks = useMemo(() => {
    const tickCount = 11;
    return Array.from({ length: tickCount }).map((_, i) => {
      const tickRatio = i / (tickCount - 1);
      const angle = startAngle + (tickRatio * totalAngle);
      const isMajor = i % 5 === 0;

      const outer = polarToCartesian(cx, cy, radius, angle);
      const inner = polarToCartesian(cx, cy, radius - (isMajor ? 15 : 8), angle);

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
    <div style={{ width: width, display: 'inline-block', position: 'relative' }}>
      <svg viewBox="0 0 300 260" style={{ width: '100%', height: 'auto', background: 'transparent' }}>
        <defs>
          {/* Glow filter for needle */}
          <filter id={`${filterId}-needleGlow`} x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="3" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>

          {/* Glow filter for target range */}
          <filter id={`${filterId}-rangeGlow`} x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="4" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>

          {/* SRM Color Gradient for target range */}
          {showSrmGradient && rangeArcData && (
            <linearGradient id={`${filterId}-srmGradient`} x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor={srmToHex(rangeArcData.minValue)} />
              <stop offset="25%" stopColor={srmToHex(rangeArcData.minValue + (rangeArcData.maxValue - rangeArcData.minValue) * 0.25)} />
              <stop offset="50%" stopColor={srmToHex(rangeArcData.minValue + (rangeArcData.maxValue - rangeArcData.minValue) * 0.5)} />
              <stop offset="75%" stopColor={srmToHex(rangeArcData.minValue + (rangeArcData.maxValue - rangeArcData.minValue) * 0.75)} />
              <stop offset="100%" stopColor={srmToHex(rangeArcData.maxValue)} />
            </linearGradient>
          )}
        </defs>

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
            stroke={showSrmGradient ? `url(#${filterId}-srmGradient)` : "#10b981"}
            strokeWidth="12"
            strokeOpacity={showSrmGradient ? "0.8" : "0.6"}
            strokeLinecap="butt"
            // filter={`url(#${filterId}-rangeGlow)`}
          />
        )}

        {/* Labels */}
        <text x={cx} y={cy + 60} textAnchor="middle" className="fill-gray-500 dark:fill-gray-400" fontSize="14" fontFamily="monospace" letterSpacing="2px">
          {label}
        </text>
        <text
          x={cx}
          y={cy + 90}
          textAnchor="middle"
          className={`${isInRange ? 'fill-gray-900 dark:fill-white' : 'fill-red-600 dark:fill-red-500'} glow-text`}
          fontSize="32"
          fontFamily="monospace"
          fontWeight="bold"
        >
          {label === "ABV" ? `${Math.round(value)}%` : Math.round(value)}
        </text>

        <text x={cx - 90} y={cy + 80} textAnchor="middle" className="fill-gray-600 dark:fill-gray-500" fontSize="12" fontFamily="monospace">{min}</text>
        <text x={cx + 90} y={cy + 80} textAnchor="middle" className="fill-gray-600 dark:fill-gray-500" fontSize="12" fontFamily="monospace">{max}</text>

        {/* Needle Group */}
        <g
          style={{
            transition: 'transform 0.6s cubic-bezier(0.34, 1.56, 0.64, 1)',
            transformOrigin: `${cx}px ${cy}px`,
            transform: `rotate(${currentAngle}deg)`
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
            style={{ transform: 'translateX(-1px)' }}
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

// --- SRM Color Utility ---

/**
 * Approximate conversion of SRM value to Hex Color.
 * Useful for passing directly to valueColor prop for color dials.
 */
export const srmToHex = (srm: number): string => {
  const srmColors: Record<number, string> = {
    1: "#FFE699", 2: "#FFD878", 3: "#FFCA5A", 4: "#FFBF42",
    5: "#FBB123", 6: "#F8A600", 7: "#F39C00", 8: "#EA8F00",
    9: "#E58500", 10: "#DE7C00", 11: "#D77200", 12: "#CF6900",
    13: "#CB6200", 14: "#C35900", 15: "#BB5100", 16: "#B54C00",
    17: "#B04500", 18: "#A63E00", 19: "#A13700", 20: "#9B3200",
    21: "#952D00", 22: "#8E2900", 23: "#882300", 24: "#821E00",
    25: "#7B1A00", 26: "#771900", 27: "#701400", 28: "#6A0E00",
    29: "#660D00", 30: "#5E0B00", 35: "#530900", 40: "#4C0500"
  };

  const rounded = Math.round(srm);
  if (rounded < 1) return "#FFE699";
  if (rounded > 40) return "#000000";

  // If exact match found
  if (srmColors[rounded]) return srmColors[rounded];

  // Fallback for gaps (simple nearest neighbor or default)
  return "#FBB123";
};