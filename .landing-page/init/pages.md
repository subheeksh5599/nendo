# Page Dependency Trees

## / (Landing Page)
Entry: `dashboard/src/App.tsx`
Dependencies:
- `dashboard/src/App.tsx` — full page: nav, hero, mobile sheet, video background
  - `lucide-react` — icons: ArrowRightCircle, Zap, LockKeyhole, Fingerprint, Menu, X
  - `framer-motion` — motion, AnimatePresence
  - `react` — useState

No other local imports. The entire page is self-contained in one file.
CSS dependencies:
- `dashboard/src/index.css` — globals + Tailwind import + CSS variables
- Tailwind v4 (auto-detected from `@import "tailwindcss"`)
