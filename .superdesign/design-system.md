# Nendo Design System

## Brand
- **Name**: Nendo
- **Tagline**: Agent RPC Firewall for Avalanche
- **Tone**: Technical, security-focused, editorial

## Fonts
- **Heading**: Helvetica Now Display Bold (loaded from CDN, fallback: sans-serif)
- **Body**: Inter (Google Fonts, weights 300-900, fallback: sans-serif)

## Colors
| Token | Hex | Usage |
|-------|-----|-------|
| Text | `#192837` | All text, nav links, brand name, icons |
| Accent | `#7342E2` | Primary CTA button background, button glow shadow |
| Login BG | `#F2F2EE` | Secondary button background |
| Sheet BG | `#CFC8C5` | Mobile menu sheet background |
| Backdrop | `rgba(25,40,55,0.35)` | Mobile menu overlay with blur(4px) |
| Overlay | `rgba(255,255,255,0.40)` | White overlay on video background |

## Typography Scale
- Brand: 20px (text-xl), bold, tracking-tight
- Hero heading: clamp(26px, 5vw, 48px), line-height 1.05, letter-spacing -0.01em
- Hero subtext: clamp(14px, 2.5vw, 18px), line-height 1.65, opacity 0.8
- Nav links: 14px (text-sm), medium weight
- Buttons: 14px (text-sm), semibold

## Spacing
- Max content width: 1280px
- Nav: px-5 sm:px-8 (20px / 32px), py-4 sm:py-5 (16px / 20px)
- Hero top: clamp(40px, 8vw, 72px)
- Hero content max-width: 560px
- Heading bottom margin: 24px
- Subtext bottom margin: 32px

## Border Radius
- All buttons: 50px (rounded-full, pill shape)

## Shadows
- CTA button glow: `0 4px 24px rgba(115,66,226,0.28)`
- Mobile sheet: `-12px 0 48px rgba(25,40,55,0.18)`

## Animations (Framer Motion)
- Entry fade-up: opacity 0→1, y 28→0, 0.6s, cubic-bezier(0.22,1,0.36,1)
- Sheet slide: x 100%→0, 0.45s in / 0.35s out
- Backdrop: opacity 0→1, 0.3s
- Hover scale: 1.04-1.05
- Tap scale: 0.96

## Components
No reusable component library. UI is inline Tailwind classes.

## Icons (Lucide React)
- Zap, LockKeyhole, Fingerprint — inline in hero heading
- ArrowRightCircle — CTA button suffix
- Menu — mobile hamburger
- X — mobile sheet close

## Layout
Single-page landing with:
1. Full-screen video background (object-cover)
2. White 40% overlay
3. Fixed top navbar (centered, 1280px max)
4. Left-aligned hero content (560px max)
5. Slide-in mobile menu sheet from right
