# @airovo/rva-node

Node.js / server adapter: resolve and render RVA assets with the WASM core.

```bash
npm i @airovo/rva-node
```

```ts
import { open, openSource, resolve, renderSvg } from "@airovo/rva-node";

const handle = await openSource("file:///assets/hero.rva"); // path | file:// | http(s) | data: | Uint8Array
const { scene, svg } = renderSvg(handle, 1280, 720);
```

---

[RVA](https://rva.airovo.tech) — Responsive Visual Asset, created by Airovo Technologies.
