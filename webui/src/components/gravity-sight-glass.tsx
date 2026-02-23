import React from 'react';

interface GravitySightGlassProps {
  /** Label displayed above the glass (e.g., "OG", "FG") */
  label: string;
  /** The gravity value to display (e.g., 1.065) */
  value: number;
  /** The color of the liquid fill (hex string) */
  liquidColor: string;
  /** Fill percentage (0-100) */
  percentage: number;
  /** Minimum target value (optional, for green range indicator) */
  targetMin?: number;
  /** Maximum target value (optional, for green range indicator) */
  targetMax?: number;
  /** Minimum value for percentage calculations (default: 1.000) */
  min?: number;
  /** Maximum value for percentage calculations (default: 1.100) */
  max?: number;
}

/**
 * Vertical "sight glass" component for displaying gravity values.
 * Renders a skeuomorphic glass tube with liquid fill level.
 */
export const GravitySightGlass: React.FC<GravitySightGlassProps> = ({
  label,
  value,
  liquidColor,
  percentage: _percentage, // Ignore passed percentage, we'll calculate it
  targetMin,
  targetMax,
  min: providedMin = 1.000,
  max: providedMax = 1.100
}) => {
  // Adjust scale to include target range if needed
  const { min, max } = React.useMemo(() => {
    let adjustedMin = providedMin;
    let adjustedMax = providedMax;

    if (targetMin !== undefined && targetMin !== null && targetMin < adjustedMin) {
      adjustedMin = Math.floor(targetMin * 1000) / 1000; // Round down to 3 decimals
    }
    if (targetMax !== undefined && targetMax !== null && targetMax > adjustedMax) {
      adjustedMax = Math.ceil(targetMax * 1000) / 1000; // Round up to 3 decimals
    }

    return { min: adjustedMin, max: adjustedMax };
  }, [providedMin, providedMax, targetMin, targetMax]);

  // Calculate percentage based on adjusted scale
  const percentage = React.useMemo(() => {
    return ((value - min) / (max - min)) * 100;
  }, [value, min, max]);

  // Calculate target range positioning
  const targetRangeStyle = React.useMemo(() => {
    if (targetMin === undefined || targetMax === undefined || targetMin === null || targetMax === null) {
      return null;
    }

    const minPercent = ((targetMin - min) / (max - min)) * 100;
    const maxPercent = ((targetMax - min) / (max - min)) * 100;
    const height = maxPercent - minPercent;

    return {
      bottom: `${minPercent}%`,
      height: `${height}%`
    };
  }, [targetMin, targetMax, min, max]);

  // Check if value is within target range
  const isInRange = React.useMemo(() => {
    if (targetMin === undefined || targetMax === undefined || targetMin === null || targetMax === null) {
      return true; // Default to in range if no range specified
    }
    return value >= targetMin && value <= targetMax;
  }, [value, targetMin, targetMax]);

  // Generate tick labels for major ticks
  const tickLabels = React.useMemo(() => {
    // Major ticks at 0%, 50%, 100%
    return [
      { position: 0, value: min },
      { position: 50, value: min + (max - min) / 2 },
      { position: 100, value: max }
    ];
  }, [min, max]);

  return (
    <div className="flex flex-col items-center font-mono">
      {/* Top Value Display */}
      <div className="mb-2 text-center">
        <div className="text-gray-500 dark:text-gray-400 text-xs tracking-widest">{label}</div>
        <div className={`${isInRange ? 'text-gray-900 dark:text-white' : 'text-red-600 dark:text-red-500'} text-xl font-bold glow-text`}>
          {value.toFixed(3)}
        </div>
      </div>

      {/* Container for glass tube and target range indicator */}
      <div className="relative flex items-center gap-1">
        {/* Target Range Box - Positioned to the left */}
        {targetRangeStyle && (
          <div className="relative h-48 w-2">
            <div
              className="absolute w-full bg-emerald-500 border-t-2 border-b-2 border-emerald-600 dark:border-emerald-400 rounded-sm"
              style={targetRangeStyle}
            />
          </div>
        )}

        {/* The Glass Tube container */}
        <div className="relative h-48 w-12 bg-gray-300 dark:bg-gray-800 rounded-full border-2 border-gray-400 dark:border-gray-700 overflow-hidden shadow-inner">
          {/* The Liquid Fill */}
          <div
            className="absolute bottom-0 w-full transition-all duration-1000 ease-out"
            style={{
              height: `${percentage}%`,
              backgroundColor: liquidColor,
              boxShadow: `0 0 15px ${liquidColor}80` // Internal glow
            }}
          />
          {/* Glass Reflection Overlay (Skeuomorphic touch) */}
          <div className="absolute inset-0 bg-gradient-to-r from-white/20 via-transparent to-black/30 pointer-events-none rounded-full"></div>

          {/* Measurement lines */}
          <div className="absolute inset-0 flex flex-col justify-between py-4 px-1 pointer-events-none opacity-50">
            {[...Array(9)].map((_, i) => (
              <div
                key={i}
                className={`h-px w-full ${i % 4 === 0 ? 'bg-gray-600 dark:bg-gray-300' : 'bg-gray-500 dark:bg-gray-600 w-1/2 mx-auto'}`}
              />
            ))}
          </div>
        </div>

        {/* Tick labels on the right */}
        <div className="absolute right-14 h-48 flex flex-col justify-between py-4">
          {tickLabels.map((tick, i) => (
            <div
              key={i}
              className="text-[8px] text-gray-600 dark:text-gray-500 font-mono"
              style={{
                position: 'absolute',
                bottom: `${tick.position}%`,
                transform: 'translateY(50%)'
              }}
            >
              {tick.value.toFixed(3)}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
