# Design Tokens & Theme

## Fonts
- Heading: 'Helvetica Now Display Bold', sans-serif (loaded from onlinewebfonts.com CDN)
- Body: 'Inter', sans-serif (Google Fonts, weights 300–900)
- Monospace (favicon only): monospace

## Colors (CSS Variables)

```css
:root {
  --font-heading: 'Helvetica Now Display Bold', sans-serif;
  --font-body: 'Inter', sans-serif;
  --color-text: #192837;
  --color-accent: #7342E2;
  --color-login-bg: #F2F2EE;
}
```

| Token | Value | Usage |
|-------|-------|-------|
| `--color-text` | `#192837` | Dark navy text, nav links, brand name |
| `--color-accent` | `#7342E2` | Purple — CTA button bg, button shadow |
| `--color-login-bg` | `#F2F2EE` | Warm off-white — secondary button bg |

## Additional Colors (hardcoded)
- Sheet background: `#CFC8C5` (warm taupe/beige)
- Sheet backdrop: `rgba(25,40,55,0.35)` with `blur(4px)`
- Sheet shadow: `-12px 0 48px rgba(25,40,55,0.18)`
- Sheet divider: `rgba(25,40,55,0.12)`
- Overlay: `bg-white/40` (40% white over video)

## Spacing & Layout
- Max width: 1280px
- Nav padding: px-5 sm:px-8 py-4 sm:py-5
- Hero top padding: clamp(40px, 8vw, 72px)
- Hero content max-width: 560px

## Typography Scale
- Brand name: text-xl font-bold tracking-tight
- Hero heading: clamp(1.65rem, 5vw, 3rem), line-height 1.05, letter-spacing -0.01em
- Hero subtext: clamp(0.9rem, 2.5vw, 1.1rem), line-height 1.65, opacity 0.8
- Nav links: text-sm font-medium
- Buttons: text-sm font-semibold

## Shadows
- CTA button: `0 4px 24px rgba(115,66,226,0.28)`

## Border Radius
- Buttons: rounded-full (50px)
- Favicon rect: rx 6

## Animations (Framer Motion)
- fadeUp: opacity 0→1, y 28→0, duration 0.6s, ease [0.22, 1, 0.36, 1]
- Delays: heading 0s, subtext 0.15s, CTA 0.3s
- Sheet: slide in from right, duration 0.45s in / 0.35s out, ease same
- Backdrop: fade in 0.3s
- Mobile nav links: staggered slide left (delay 0.18 + i*0.07, duration 0.4s)
- CTA hover: scale 1.04 + brightness 1.1
- CTA tap: scale 0.96
- Nav link hover: opacity → 0.65, duration 0.2s
- Button hover: scale 1.05

## Full globals.css

```css
@import "tailwindcss";

:root {
  --font-heading: 'Helvetica Now Display Bold', sans-serif;
  --font-body: 'Inter', sans-serif;
  --color-text: #192837;
  --color-accent: #7342E2;
  --color-login-bg: #F2F2EE;
}

html {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
```

## Tech Stack
- React 19 + TypeScript
- Vite 6
- Tailwind CSS v4 (no config file — uses CSS-first config)
- Framer Motion 12
- Lucide React icons
- No component library (shadcn, MUI, etc.)
