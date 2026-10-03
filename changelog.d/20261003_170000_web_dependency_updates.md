---
bump: minor
---

### Changed

- Web app dependencies updated to their latest releases (React 19, Vite 8, Vitest 5, TypeScript 7, i18next 26, KaTeX 0.19, links-notation 0.22, lino-objects-codec 0.8, jsdom 30, …); `vite-plugin-top-level-await` and `@types/katex` are no longer needed (#223).
- CI runs on Node.js 24, since several updated web packages require Node.js 22 or newer (#223).
