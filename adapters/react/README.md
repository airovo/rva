# @airovo/rva-react

React wrapper for the `<rva-image>` web component.

```bash
npm i @airovo/rva-react
```

```tsx
import { RVAImage } from "@airovo/rva-react";

<RVAImage
  src="/hero.rva"
  alt="Hero"
  onRender={(scene) => setStatus(scene.topology)}
  onCta={(cta) => track(cta.id)}
  ctaHandlers={{ "shop-now": () => navigate("/product") }}
/>
```

Peer dependency: `react >= 18`.

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, created by Airovo Technologies.
