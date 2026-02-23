import React, { type ComponentProps } from 'react';

// --- Types ---

interface GrainIconProps extends ComponentProps<'svg'> {
  /**
   * Hex color string representing the malt color (SRM).
   * Defaults to Pale Malt (#F2C94C).
   */
  srmColor?: string;
}

// --- Components ---

export const GrainIcon = ({
  srmColor = "#F2C94C",
  className = "",
  ...props
}: GrainIconProps) => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={`size-6 stroke-1 stroke-slate-500 dark:stroke-slate-500 ${className}`}
      {...props}
    >
      {/* Central Stem */}
      <path d="M12 3v18" />

      {/* Kernels group - Dynamic fill from prop */}
      <g style={{ fill: srmColor }}>
        <path d="M12 3c0 0 1.5 2 1.5 3S12 9 12 9s-1.5-2-1.5-3S12 3 12 3z" />
        <path d="M12 8c0 0 2.5 1.5 2.5 3.5S12 15 12 15s-2.5-1.5-2.5-3.5S12 8 12 8z" />
        <path d="M12 14c0 0 2.5 1.5 2.5 3.5S12 21 12 21s-2.5-1.5-2.5-3.5S12 14 12 14z" />
        {/* Side Kernels */}
        <path d="M9.5 6c0 0-2 1.5-2 3.5S9.5 13 9.5 13l2.5-2V8L9.5 6z" />
        <path d="M14.5 6c0 0 2 1.5 2 3.5S14.5 13 14.5 13l-2.5-2V8l2.5-2z" />
        <path d="M9.5 12c0 0-2 1.5-2 3.5S9.5 19 9.5 19l2.5-2v-3l-2.5-2z" />
        <path d="M14.5 12c0 0 2 1.5 2 3.5S14.5 19 14.5 19l-2.5-2v-3l2.5-2z" />
      </g>
    </svg>
  );
};

export const HopIcon = ({ className = "", ...props }: ComponentProps<'svg'>) => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      strokeLinecap="round"
      strokeLinejoin="round"
      // Default hop green fill, overridable via className
      className={`size-6 stroke-1 fill-[#5F8C3F] stroke-slate-500 dark:stroke-slate-500 ${className}`}
      {...props}
    >
      {/* Stem at top - no fill */}
      <path d="M12 2c0 3 1.5 4 1.5 4" fill="none" />
      
      {/* Main Cone Body */}
      <path d="M13.5 6C16 6.5 19 9 19 12c0 3-2 5.5-4 7.5-2 2-3 2.5-3 2.5s-1-.5-3-2.5C7 17.5 5 15 5 12c0-3 3-5.5 5.5-6" />
      <path d="M12 6c-2 2-3 5-3 7.5 0 2 1.5 4 3 6" />
      <path d="M12 6c2 2 3 5 3 7.5 0 2-1.5 4-3 6" />
      <path d="M9 13.5C7 12 7 10 8.5 8" />
      <path d="M15 13.5C17 12 17 10 15.5 8" />
      <path d="M8.5 17c-1.5-1.5-2-3-1-5" />
      <path d="M15.5 17c1.5-1.5 2-3 1-5" />
    </svg>
  );
};

export const YeastIcon = ({ className = "", ...props }: ComponentProps<'svg'>) => {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      strokeLinecap="round"
      strokeLinejoin="round"
      // Default beige fill, overridable via className
      className={`size-6 stroke-1 fill-[#F2E6D0] stroke-slate-500 dark:stroke-slate-500 ${className}`}
      {...props}
    >
      {/* Main Cluster */}
      <path d="M16.5 4.5a3 3 0 1 1-2.8 4.3c-1 .8-2.5 1.5-3.7 2.7C8 13.5 6.5 15 7 18c.4 2.4 2.3 4 4.5 4 2.6 0 4.5-2 5-4 .3-1.4 0-2.7-.7-3.8 1.2-1 2.3-2 2.8-3.7.4-1.3.2-2.8-.7-4-1.1-1.3-2.4-1.8-1.4-2z"/>
      
      {/* Small Cells */}
      <circle cx="5.5" cy="8.5" r="2.5" />
      <circle cx="18.5" cy="19.5" r="2" />
      
      {/* Highlights - no fill */}
      <path d="M6 7a2 2 0 0 1 1 2" strokeWidth="1" fill="none" />
      <path d="M9 19a3 3 0 0 1 3-3" strokeWidth="1" fill="none" />
    </svg>
  );
};