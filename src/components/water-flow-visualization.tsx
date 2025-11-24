import React from 'react';
import type { CalculatedStage } from '@/lib/water-calculations';

// --- Constants & Layout ---

const MAX_VISUAL_VOL = 15;
const TANK_WIDTH = 120;
const CYLINDER_HEIGHT = 200;
const CONE_DROP = 70;
const BASELINE_Y = 300;
const SVG_HEIGHT = 450;
const SVG_WIDTH = 1100;
const PX_PER_GAL = CYLINDER_HEIGHT / MAX_VISUAL_VOL;

// --- Shapes Logic (SVG Paths) ---

const getTankPath = (shape: string, w: number, h: number): string => {
  switch (shape) {
    case 'dome': // Mash Tun
      return `M0,${h} V40 Q${w/2},0 ${w},40 V${h} Z`;

    case 'chimney': // Kettle
      const neckW = w * 0.4;
      const neckX = (w - neckW) / 2;
      return `M0,${h} V40 H${neckX} V5 H${neckX + neckW} V40 H${w} V${h} Z`;

    case 'conical': // Fermenter
      return `M0,0 H${w} V${h} L${w/2},${h + CONE_DROP} L0,${h} Z`;

    case 'keg': // Packaging
      return `
        M10,5
        Q0,5 0,15 V${h-15} Q0,${h-5} 10,${h-5}
        H${w-10}
        Q${w},${h-5} ${w},${h-15} V15 Q${w},5 ${w-10},5
        Z
        M10,5 V0 H${w-10} V5
        M10,${h-5} V${h} H${w-10} V${h-5}
      `;

    case 'rect': // Source Tank
    default:
      return `M0,0 H${w} V${h} H0 Z`;
  }
};

interface WaterFlowVisualizationProps {
  stages: CalculatedStage[];
  beerColor?: string;
}

export function WaterFlowVisualization({
  stages,
  beerColor = "#F59E0B"
}: WaterFlowVisualizationProps) {
  return (
    <svg viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`} className="w-full h-auto">
      <defs>
        {/* Liquid Gradient */}
        <linearGradient id="liquidGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="white" stopOpacity="0.2" />
          <stop offset="25%" stopColor="white" stopOpacity="0.05" />
          <stop offset="50%" stopColor="white" stopOpacity="0" />
          <stop offset="75%" stopColor="black" stopOpacity="0.1" />
          <stop offset="100%" stopColor="black" stopOpacity="0.3" />
        </linearGradient>

        {/* Clip Paths for Shapes */}
        {stages.map((stage) => (
          <clipPath key={`clip-${stage.id}`} id={`clip-${stage.id}`}>
            <path d={getTankPath(stage.shape, TANK_WIDTH, CYLINDER_HEIGHT)} />
          </clipPath>
        ))}
      </defs>

      {/* Render Loop */}
      {stages.map((stage, i) => {
        const xPos = 50 + (i * 200);
        const tankTopY = BASELINE_Y - CYLINDER_HEIGHT;

        const isConical = stage.shape === 'conical';
        const totalTankHeight = isConical ? CYLINDER_HEIGHT + CONE_DROP : CYLINDER_HEIGHT;

        // Calculate Liquid
        const rawHeight = stage.volumeIn * PX_PER_GAL;
        const maxVisualHeight = isConical ? CYLINDER_HEIGHT + CONE_DROP : CYLINDER_HEIGHT;
        const liquidHeight = isConical
          ? Math.max(0, Math.min(rawHeight * (totalTankHeight / CYLINDER_HEIGHT), maxVisualHeight))
          : Math.max(0, Math.min(rawHeight, maxVisualHeight));

        const liquidColor = stage.isSource ? "#3B82F6" : beerColor;
        const nextPipeColor = stage.isSource ? "#3B82F6" : beerColor;

        return (
          <g key={stage.id}>

            {/* PIPE to Next Tank */}
            {i < stages.length - 1 && (
              <g>
                <path
                  d={`M${xPos + TANK_WIDTH - 2},${BASELINE_Y - 20} H${xPos + TANK_WIDTH + 80}`}
                  stroke="#334155" strokeWidth="12" fill="none"
                />
                <path
                  d={`M${xPos + TANK_WIDTH},${BASELINE_Y - 20} H${xPos + TANK_WIDTH + 80}`}
                  stroke={nextPipeColor} strokeWidth="6" strokeOpacity="0.6" fill="none" strokeDasharray="5 5"
                >
                  <animate attributeName="stroke-dashoffset" from="10" to="0" dur="1s" repeatCount="indefinite" />
                </path>
              </g>
            )}

            {/* TANK GROUP */}
            <g transform={`translate(${xPos}, ${tankTopY})`}>

              {/* Tank Shell */}
              <path
                d={getTankPath(stage.shape, TANK_WIDTH, CYLINDER_HEIGHT)}
                fill="#1e293b"
                fillOpacity="0.5"
                stroke="#475569"
                strokeWidth="3"
              />

              {/* Liquid (Clipped) */}
              <g clipPath={`url(#clip-${stage.id})`}>
                <rect
                  x="0"
                  y={isConical ? (CYLINDER_HEIGHT + CONE_DROP - liquidHeight) : (CYLINDER_HEIGHT - liquidHeight)}
                  width={TANK_WIDTH}
                  height={liquidHeight + 20}
                  fill={liquidColor}
                  className="transition-all duration-700 ease-out"
                />
                <rect
                  x="0"
                  y={isConical ? (CYLINDER_HEIGHT + CONE_DROP - liquidHeight) : (CYLINDER_HEIGHT - liquidHeight)}
                  width={TANK_WIDTH}
                  height={liquidHeight + 20}
                  fill="url(#liquidGradient)"
                  style={{ mixBlendMode: 'overlay' }}
                />
                {/* Meniscus Line */}
                <path
                  d={`M0,${isConical ? (CYLINDER_HEIGHT + CONE_DROP - liquidHeight) : (CYLINDER_HEIGHT - liquidHeight)} H${TANK_WIDTH}`}
                  stroke="white" strokeWidth="2" strokeOpacity="0.4"
                />
              </g>

              {/* Labels */}
              <text x={TANK_WIDTH/2} y="-15" textAnchor="middle" fill="#94a3b8" fontSize="12" fontWeight="bold" letterSpacing="1px">
                {stage.label.toUpperCase()}
              </text>

              <text x={TANK_WIDTH/2} y={CYLINDER_HEIGHT/2} textAnchor="middle" fill="white" fontSize="18" fontWeight="800" style={{ textShadow: '0 2px 4px rgba(0,0,0,0.5)' }}>
                {stage.volumeIn.toFixed(2)}
              </text>
            </g>

            {/* LOSS DRAINS */}
            {stage.totalLoss > 0 && (
              <g className="transition-all duration-500 ease-out">
                <g transform={`translate(${xPos + (TANK_WIDTH/2)}, ${isConical ? BASELINE_Y + CONE_DROP : BASELINE_Y})`}>
                  <rect x="-4" y="0" width="8" height="15" fill="#334155" />
                  <rect
                    x="-3" y="15"
                    width="6"
                    height={Math.min(100, 20 + (stage.totalLoss * 30))}
                    fill={liquidColor}
                    opacity="0.7"
                    rx="3"
                  />
                  <text
                    x="0"
                    y={15 + Math.min(100, 20 + (stage.totalLoss * 30)) + 15}
                    textAnchor="middle"
                    fill="#ef4444"
                    fontSize="11"
                    fontWeight="bold"
                  >
                    -{stage.totalLoss.toFixed(2)}
                  </text>
                </g>
              </g>
            )}

          </g>
        );
      })}
    </svg>
  );
}
