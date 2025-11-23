import React from 'react';

interface ScrewProps {
  /** Position in top-left corner */
  tl?: boolean;
  /** Position in top-right corner */
  tr?: boolean;
  /** Position in bottom-left corner */
  bl?: boolean;
  /** Position in bottom-right corner */
  br?: boolean;
}

/**
 * Decorative screw component for retro panel aesthetic.
 * Positions itself absolutely in one of four corners.
 */
export const Screw: React.FC<ScrewProps> = ({ tl, tr, bl, br }) => {
  const classes = `absolute w-3 h-3 rounded-full bg-gray-400 dark:bg-gray-700 border border-gray-500 dark:border-gray-900 shadow-inner flex items-center justify-center`;
  const pos = tl ? 'top-2 left-2' : tr ? 'top-2 right-2' : bl ? 'bottom-2 left-2' : 'bottom-2 right-2';

  return (
    <div className={`${classes} ${pos}`}>
      <div className="w-full h-px bg-gray-600 dark:bg-gray-900 rotate-45"></div>
    </div>
  );
};
