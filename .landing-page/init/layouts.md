# Layout Components

No shared layout components exist. The entire page is rendered in a single App.tsx component.

## Structure
- Root div: full-width min-h-screen with video background
- Nav: max-w-[1280px] centered, flex row, brand + desktop links + buttons + mobile hamburger
- Hero: max-w-[1280px] centered, max-w-[560px] content block
- Mobile Sheet: animated slide-in from right, overlay backdrop

## Entry Point

`/home/arch/nendo/dashboard/src/main.tsx`:
```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
```
