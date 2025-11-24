import React, { useState, useMemo } from 'react';

// --- Types ---

type LossType = 'flat' | 'rate';
type TankShape = 'rect' | 'dome' | 'chimney' | 'conical' | 'keg';

interface Loss {
  id: string;
  label: string;
  value: number;
  type: LossType;
  unit: string;
}

interface Stage {
  id: string;
  label: string;
  shape: TankShape;
  losses: Loss[];
}

interface CalculatedStage extends Stage {
  volumeIn: number;
  volumeOut: number;
  totalLoss: number;
  isSource?: boolean;
}

interface CalculationResult {
  totalWaterNeeded: number;
  stages: CalculatedStage[];
}

// --- Constants & Layout ---

const MAX_VISUAL_VOL = 15; 
const TANK_WIDTH = 120;
const CYLINDER_HEIGHT = 200; // The standard height of the "body"
const CONE_DROP = 70; // How far the cone hangs down below the baseline
const BASELINE_Y = 300; // The y-coordinate where standard tanks sit
const SVG_HEIGHT = 450; // Increased to fit the cone and the bottom drains
const SVG_WIDTH = 1100;
const PX_PER_GAL = CYLINDER_HEIGHT / MAX_VISUAL_VOL;

// --- Shapes Logic (SVG Paths) ---

const getTankPath = (shape: TankShape, w: number, h: number): string => {
  switch (shape) {
    case 'dome': // Mash Tun
      return `M0,${h} V40 Q${w/2},0 ${w},40 V${h} Z`;
    
    case 'chimney': // Kettle
      const neckW = w * 0.4;
      const neckX = (w - neckW) / 2;
      return `M0,${h} V40 H${neckX} V5 H${neckX + neckW} V40 H${w} V${h} Z`;
    
    case 'conical': // Fermenter (Extends BELOW h)
      // h here represents the cylinder bottom (baseline). 
      // We draw the cone extending downwards from h.
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

const INITIAL_STAGES: Stage[] = [
  {
    id: "mash",
    label: "Mash Tun",
    shape: "dome",
    losses: [
      { id: "grainAbs", label: "Grain Absorption", value: 1.25, type: "flat", unit: "gal" },
      { id: "tunDead", label: "Tun Deadspace", value: 0.25, type: "flat", unit: "gal" }
    ]
  },
  {
    id: "kettle",
    label: "Boil Kettle",
    shape: "chimney",
    losses: [
      { id: "boilOff", label: "Boil Off Rate", value: 1.0, type: "rate", unit: "gal/hr" },
      { id: "trub", label: "Trub / Hop Loss", value: 0.5, type: "flat", unit: "gal" }
    ]
  },
  {
    id: "fermenter",
    label: "Fermenter",
    shape: "conical",
    losses: [
      { id: "trub_ferm", label: "Yeast/Cake Loss", value: 0.5, type: "flat", unit: "gal" }
    ]
  },
  {
    id: "packaging",
    label: "Kegging",
    shape: "keg",
    losses: [
      { id: "lines", label: "Transfer Loss", value: 0.1, type: "flat", unit: "gal" }
    ]
  }
];

// --- Component ---

const WaterCalculator: React.FC = () => {
  const [targetVolume, setTargetVolume] = useState<number>(5.0);
  const [boilTime, setBoilTime] = useState<number>(60);
  const [beerColor, setBeerColor] = useState<string>("#F59E0B");
  const [stages, setStages] = useState<Stage[]>(INITIAL_STAGES);

  // --- Calculation Logic ---
  const data = useMemo((): CalculationResult => {
    let currentVol = targetVolume;
    const processedStages: CalculatedStage[] = [];

    [...stages].reverse().forEach(stage => {
      let stageLossTotal = 0;
      stage.losses.forEach(loss => {
        let val = loss.value;
        if (loss.type === 'rate') val = val * (boilTime / 60);
        stageLossTotal += val;
      });

      const volIn = currentVol + stageLossTotal;
      processedStages.push({
        ...stage,
        volumeIn: volIn,
        volumeOut: currentVol,
        totalLoss: stageLossTotal,
      });
      currentVol = volIn;
    });

    const orderedStages = processedStages.reverse();

    const sourceTank: CalculatedStage = {
      id: 'source',
      label: 'Strike Water',
      shape: 'rect',
      losses: [],
      volumeIn: currentVol,
      volumeOut: currentVol,
      totalLoss: 0,
      isSource: true
    };

    return {
      totalWaterNeeded: currentVol,
      stages: [sourceTank, ...orderedStages]
    };
  }, [targetVolume, boilTime, stages]);

  // Handlers
  const handleLossChange = (stageIndex: number, lossIndex: number, val: number) => {
    const realIndex = stageIndex - 1; 
    if (realIndex < 0) return; 
    const newStages = [...stages];
    newStages[realIndex].losses[lossIndex].value = val;
    setStages(newStages);
  };

  const handleAddCustomLoss = (stageIndex: number) => {
    const realIndex = stageIndex - 1; 
    if (realIndex < 0) return;
    const newStages = [...stages];
    newStages[realIndex].losses.push({
      id: `custom_${Date.now()}`,
      label: "Custom Loss",
      value: 0.1,
      type: "flat",
      unit: "gal"
    });
    setStages(newStages);
  };

  return (
    <div className="bg-slate-900 min-h-screen text-slate-200 p-4 xl:p-8 font-sans">
      
      {/* HEADER */}
      <header className="mb-6 flex flex-col md:flex-row justify-between items-end border-b border-slate-700 pb-4">
        <div>
            <h1 className="text-3xl font-extrabold text-white tracking-tight">BREWFLOW</h1>
            <p className="text-slate-400 text-sm font-medium">Process & Water Visualizer</p>
        </div>
        
        <div className="flex items-end gap-8 mt-4 md:mt-0">
             <div className="flex flex-col items-end">
                <label className="text-[10px] uppercase font-bold text-slate-500 mb-1">Beer Color</label>
                <div className="flex gap-2">
                    {['#F59E0B', '#D97706', '#92400E', '#78350F', '#000000'].map(c => (
                        <button 
                            key={c} 
                            onClick={() => setBeerColor(c)}
                            className={`w-6 h-6 rounded-full border-2 ${beerColor === c ? 'border-white' : 'border-transparent'}`}
                            style={{ backgroundColor: c }}
                        />
                    ))}
                </div>
            </div>

            <div className="text-right">
                <div className="text-4xl font-black text-blue-400 tabular-nums">
                    {data.totalWaterNeeded.toFixed(2)} <span className="text-xl text-slate-500">gal</span>
                </div>
                <div className="text-[10px] uppercase tracking-widest text-slate-500 font-bold">Total Start Volume</div>
            </div>
        </div>
      </header>

      <main className="grid grid-cols-1 2xl:grid-cols-12 gap-6">
        
        {/* --- VISUALIZATION --- */}
        <div className="2xl:col-span-8 bg-slate-800 rounded-3xl p-4 shadow-2xl border border-slate-700 relative overflow-hidden flex items-center justify-center min-h-[450px]">
          
          <svg viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`} className="w-full h-auto">
            <defs>
                {/* 1. Liquid Gradient */}
                <linearGradient id="liquidGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stopColor="white" stopOpacity="0.2" />
                    <stop offset="25%" stopColor="white" stopOpacity="0.05" />
                    <stop offset="50%" stopColor="white" stopOpacity="0" />
                    <stop offset="75%" stopColor="black" stopOpacity="0.1" />
                    <stop offset="100%" stopColor="black" stopOpacity="0.3" />
                </linearGradient>

                 {/* 2. Clip Paths for Shapes */}
                 {data.stages.map((stage) => (
                     <clipPath key={`clip-${stage.id}`} id={`clip-${stage.id}`}>
                        <path d={getTankPath(stage.shape, TANK_WIDTH, CYLINDER_HEIGHT)} />
                     </clipPath>
                 ))}
            </defs>

            {/* --- Render Loop --- */}
            {data.stages.map((stage, i) => {
              const xPos = 50 + (i * 200);
              const tankTopY = BASELINE_Y - CYLINDER_HEIGHT;
              
              // Visual Params
              const isConical = stage.shape === 'conical';
              const totalTankHeight = isConical ? CYLINDER_HEIGHT + CONE_DROP : CYLINDER_HEIGHT;

              // Calculate Liquid
              // For conical tanks, we need to normalize the fill level so it appears visually
              // consistent with other tanks. The cone extends the visual height but we want
              // the same volume to show a similar fill percentage.
              // Normalize: use the cylinder height as the reference, not the total with cone.
              const rawHeight = stage.volumeIn * PX_PER_GAL;
              const maxVisualHeight = isConical ? CYLINDER_HEIGHT + CONE_DROP : CYLINDER_HEIGHT;
              // For conical, scale the liquid height to fill proportionally within the total visual height
              const liquidHeight = isConical
                ? Math.max(0, Math.min(rawHeight * (totalTankHeight / CYLINDER_HEIGHT), maxVisualHeight))
                : Math.max(0, Math.min(rawHeight, maxVisualHeight));
              
              // Color Logic
              const liquidColor = stage.isSource ? "#3B82F6" : beerColor; 
              const nextPipeColor = stage.isSource ? "#3B82F6" : beerColor;

              return (
                <g key={stage.id}>
                  
                  {/* PIPE to Next Tank */}
                  {i < data.stages.length - 1 && (
                      <g>
                        {/* Outer Pipe */}
                        <path 
                            d={`M${xPos + TANK_WIDTH - 2},${BASELINE_Y - 20} H${xPos + TANK_WIDTH + 80}`} 
                            stroke="#334155" strokeWidth="12" fill="none" 
                        />
                        {/* Inner Liquid */}
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
                      
                      {/* 1. Tank Shell (Glass) */}
                      <path 
                        d={getTankPath(stage.shape, TANK_WIDTH, CYLINDER_HEIGHT)}
                        fill="#1e293b" 
                        fillOpacity="0.5"
                        stroke="#475569" 
                        strokeWidth="3" 
                      />

                      {/* 2. Liquid (Clipped) */}
                      <g clipPath={`url(#clip-${stage.id})`}>
                        {/* We draw the rect from the bottom up.
                           If conical, the rect needs to start lower.
                           y = TotalShapeHeight - liquidHeight 
                           If conical, shape height is taller, so y starts lower relative to cylinder top.
                        */}
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

                       {/* 3. Labels */}
                       <text x={TANK_WIDTH/2} y="-15" textAnchor="middle" fill="#94a3b8" fontSize="12" fontWeight="bold" letterSpacing="1px">
                         {stage.label.toUpperCase()}
                       </text>
                       
                       {/* Volume text sits in middle of cylinder body */}
                       <text x={TANK_WIDTH/2} y={CYLINDER_HEIGHT/2} textAnchor="middle" fill="white" fontSize="18" fontWeight="800" style={{ textShadow: '0 2px 4px rgba(0,0,0,0.5)' }}>
                         {stage.volumeIn.toFixed(2)}
                       </text>
                  </g>

                  {/* LOSS DRAINS (Bottom of Tank) */}
                  {stage.totalLoss > 0 && (
                    <g className="transition-all duration-500 ease-out">
                        {/* Calculate Bottom coordinate:
                           If standard: BASELINE_Y
                           If conical: BASELINE_Y + CONE_DROP
                        */}
                        <g transform={`translate(${xPos + (TANK_WIDTH/2)}, ${isConical ? BASELINE_Y + CONE_DROP : BASELINE_Y})`}>
                            
                            {/* Vertical Drop Pipe */}
                            <rect x="-4" y="0" width="8" height="15" fill="#334155" />
                            
                            {/* The Loss Stream (Growing Downwards) */}
                            <rect 
                                x="-3" y="15" 
                                width="6" 
                                height={Math.min(100, 20 + (stage.totalLoss * 30))} 
                                fill={liquidColor} 
                                opacity="0.7"
                                rx="3"
                            />
                            
                            {/* Text Label (Below the stream) */}
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
        </div>

        {/* --- CONTROLS --- */}
        <div className="2xl:col-span-4 flex flex-col gap-4 h-[600px] overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-slate-700">
            
            {/* Batch Settings */}
            <div className="bg-slate-800 p-4 rounded-xl border border-slate-700 shadow-md">
                <div className="grid grid-cols-2 gap-4">
                    <div>
                        <label className="block text-[10px] uppercase text-slate-500 font-bold mb-1">Packaging Target</label>
                        <div className="relative">
                            <input 
                                type="number" 
                                value={targetVolume} 
                                onChange={(e) => setTargetVolume(Number(e.target.value))} 
                                className="w-full bg-slate-900 border border-slate-600 rounded p-2 text-white font-mono focus:border-blue-500 outline-none" 
                            />
                            <span className="absolute right-3 top-2 text-slate-500 text-xs">gal</span>
                        </div>
                    </div>
                    <div>
                        <label className="block text-[10px] uppercase text-slate-500 font-bold mb-1">Boil Time</label>
                        <div className="relative">
                            <input 
                                type="number" 
                                value={boilTime} 
                                onChange={(e) => setBoilTime(Number(e.target.value))} 
                                className="w-full bg-slate-900 border border-slate-600 rounded p-2 text-white font-mono focus:border-blue-500 outline-none" 
                            />
                            <span className="absolute right-3 top-2 text-slate-500 text-xs">min</span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Stages Controls */}
            {data.stages.map((stage, i) => {
                if(stage.isSource) return null; 

                return (
                <div key={stage.id} className="bg-slate-800 p-4 rounded-xl border-l-[4px] border-slate-700 relative" style={{ borderLeftColor: stage.id === 'packaging' ? '#10B981' : beerColor }}>
                    <div className="flex justify-between items-center mb-4">
                        <h3 className="font-bold text-white text-sm">{stage.label}</h3>
                        <button 
                            onClick={() => handleAddCustomLoss(i)} 
                            className="text-[10px] bg-slate-700 hover:bg-slate-600 text-slate-300 px-2 py-1 rounded"
                        >
                            + LOSS
                        </button>
                    </div>
                    
                    <div className="space-y-4">
                        {stage.losses.map((loss, lIndex) => (
                            <div key={loss.id}>
                                <div className="flex justify-between text-xs mb-1">
                                    <span className="text-slate-400">{loss.label}</span>
                                    <span className="text-slate-200 font-mono">
                                        {loss.value} <span className="text-slate-600">{loss.unit}</span>
                                    </span>
                                </div>
                                <input 
                                    type="range" 
                                    min="0" 
                                    max="2.0" 
                                    step="0.05" 
                                    value={loss.value}
                                    onChange={(e) => handleLossChange(i, lIndex, Number(e.target.value))}
                                    className="w-full h-1 bg-slate-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
                                />
                            </div>
                        ))}
                        {stage.losses.length === 0 && <div className="text-xs text-slate-600 italic">No losses configured.</div>}
                    </div>
                </div>
            )})}
        </div>

      </main>
    </div>
  );
};

export default WaterCalculator;