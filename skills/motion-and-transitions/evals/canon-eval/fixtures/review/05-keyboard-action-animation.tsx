// Fixture 05 — planted defect: animation on a 100+/day keyboard-initiated action.
// Defect class: Purpose & frequency.
// Expected finding: command-palette toggle is a keyboard shortcut fired hundreds of
// times a day — it should have NO animation ("100+ times/day -> No animation. Ever.").
// This component fades + scales the palette in/out on every toggle.

import { AnimatePresence, motion } from 'framer-motion';

export function CommandPalette({ open }: { open: boolean }) {
  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className="command-palette"
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          transition={{ duration: 0.18 }}
        >
          {/* palette contents */}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
