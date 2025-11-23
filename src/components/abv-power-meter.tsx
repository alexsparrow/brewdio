import React from 'react';

interface AbvPowerMeterProps {
  /** The ABV value to display (e.g., 6.7 for 6.7%) */
  value: number;
  /** Maximum value for full scale (default: 12) */
  maxValue?: number;
}

/**
 * Horizontal LED-style bar meter for displaying ABV.
 * Features color-coded segments (green → amber → red) and retro LED aesthetic.
 */
export const AbvPowerMeter: React.FC<AbvPowerMeterProps> = ({ value, maxValue = 12 }) => {
  const segments = 20;
  const filledSegments = Math.min(Math.floor((value / maxValue) * segments), segments);

  const getSegmentColor = (index: number) => {
    if (index < filledSegments) {
      if (index > segments * 0.8) return "bg-red-500 shadow-[0_0_8px_#ef4444]";
      if (index > segments * 0.5) return "bg-amber-500 shadow-[0_0_8px_#f59e0b]";
      return "bg-emerald-500 shadow-[0_0_8px_#10b981]";
    }
    return "bg-gray-300 dark:bg-gray-800 border border-gray-400 dark:border-gray-700 opacity-50"; // Unlit segment
  };

  return (
    <div className="w-full p-3 bg-white/70 dark:bg-gray-900/50 rounded-lg border border-gray-300 dark:border-gray-700 font-mono relative overflow-hidden">
      {/* Background Scanline effect */}
      <div className="absolute inset-0 pointer-events-none bg-[url('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAADCAYAAABS3WWCAAAAAlwSFlzAAALEwAACxMBAJqcGAAAAA1JREFUCJljYGZgYAAAAAYAAwa1YAAAAABJRU5ErkJggg==')] opacity-10 mix-blend-overlay"></div>

      <div className="flex justify-between items-end mb-2">
        <span className="text-gray-600 dark:text-gray-400 tracking-widest text-sm">EST. ABV</span>
        <span className="text-2xl text-gray-900 dark:text-white font-bold glow-text">{value.toFixed(1)}%</span>
      </div>

      {/* The LED Segment Bar */}
      <div className="flex gap-1 h-5">
        {[...Array(segments)].map((_, i) => (
          <div
            key={i}
            className={`flex-1 rounded-sm transition-all duration-300 ${getSegmentColor(i)}`}
          />
        ))}
      </div>

      <div className="flex justify-between text-xs text-gray-500 dark:text-gray-500 mt-1">
        <span>0%</span>
        <span>{maxValue / 2}%</span>
        <span>{maxValue}%+</span>
      </div>
    </div>
  );
};
