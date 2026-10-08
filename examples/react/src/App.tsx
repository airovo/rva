import { useMemo, useRef, useState } from "react";
import { RVAImage, type CtaRegion } from "@airovo/rva-react";

// The <rva-image> source escape hatches (set from a ref; JSX can't pass objects).
interface RvaImageElement extends HTMLElement {
  bytes: Uint8Array | ArrayBuffer | Blob | null;
  reload(): void;
}

// The RVA Torture Corpus — 15 adversarial fixtures, served from /fixtures/*.rva.
const FIXTURES = [
  "01-basic-hero",
  "02-long-headline",
  "03-multiple-subjects",
  "04-product-and-person",
  "05-overlapping-focal-regions",
  "06-extreme-landscape",
  "07-extreme-portrait",
  "08-decoration-degradation",
  "09-topology-boundary",
  "10-impossible-constraints",
  "11-many-elements",
  "12-missing-resource",
  "13-large-resources",
  "14-unicode-text",
  "15-fallback",
];

// The CTA region declared by fixture 01-basic-hero.
const SHOP_CTA = "shop-cta";

export function App() {
  const ref = useRef<HTMLElement>(null);
  const fileInput = useRef<HTMLInputElement>(null);
  const [fixture, setFixture] = useState(FIXTURES[0]);
  const [localName, setLocalName] = useState<string | null>(null);
  const [status, setStatus] = useState("resolving…");
  const [ctas, setCtas] = useState<CtaRegion[]>([]);
  const [lastCta, setLastCta] = useState<string | null>(null);
  const [shopHits, setShopHits] = useState(0);
  const [showCta, setShowCta] = useState(true);

  const pick = (id: string) => {
    const element = ref.current as RvaImageElement | null;
    // Drop any locally-loaded bytes so the fixture URL takes over again.
    if (element) element.bytes = null;
    setLocalName(null);
    if (id === fixture) {
      element?.reload();
      return;
    }
    setStatus("resolving…");
    setCtas([]);
    setLastCta(null);
    setFixture(id);
  };

  // A browser cannot read file:// URLs. Hosts hand bytes to the element instead,
  // which is exactly what a file picker (or a Node/RN loader) provides.
  const loadLocalFile = async (file: File) => {
    const element = ref.current as RvaImageElement | null;
    if (!element) return;
    setStatus("loading local file…");
    setCtas([]);
    setLastCta(null);
    element.bytes = new Uint8Array(await file.arrayBuffer());
    element.reload();
    setLocalName(file.name);
  };

  const ctaSummary = useMemo(() => {
    if (ctas.length === 0) return "no CTA regions";
    return `CTA ${ctas.map((region) => region.id).join(", ")}`;
  }, [ctas]);

  return (
    <div className="wrap">
      <h1 style={{ fontSize: 18, margin: 0 }}>RVA — Torture Corpus (React)</h1>
      <p>
        All 15 adversarial fixtures. Fixture <b>01-basic-hero</b> declares a CTA
        region <code>shop-cta</code> bound to the product. Resize the window, switch
        fixtures, then click the highlighted region.
      </p>

      <div className="picker">
        {FIXTURES.map((id) => (
          <button
            key={id}
            type="button"
            className={id === fixture ? "chip active" : "chip"}
            onClick={() => pick(id)}
          >
            {id}
          </button>
        ))}
      </div>

      <div className="row">
        <label className="toggle">
          <input
            type="checkbox"
            checked={showCta}
            onChange={(event) => setShowCta(event.target.checked)}
          />
          Show CTA regions
        </label>
        <button type="button" className="chip" onClick={() => fileInput.current?.click()}>
          Load .rva file…
        </button>
        <input
          ref={fileInput}
          type="file"
          accept=".rva"
          hidden
          onChange={(event) => {
            const file = event.target.files?.[0];
            if (file) void loadLocalFile(file);
            event.target.value = "";
          }}
        />
        {localName && <span className="badge">local: {localName}</span>}
      </div>

      <div className="stage">
        {/* No ref-juggling or addEventListener: CTA handling is a prop. */}
        <RVAImage
          ref={ref}
          src={`/fixtures/${fixture}.rva`}
          alt={`RVA fixture ${fixture}`}
          className="hero"
          onRender={(scene) => {
            const hidden = scene.hidden?.length ? ` · hidden ${scene.hidden.length}` : "";
            setStatus(
              `topology ${scene.topology} · ${scene.width}×${scene.height} · ` +
                `viability ${scene.viability.toFixed(2)}` +
                `${scene.degraded ? " · degraded" : ""}${hidden}`
            );
          }}
          onCtaRegions={(regions) => setCtas(regions.filter((region) => region.visible))}
          onCta={(cta) => setLastCta(cta.id)}
          ctaHandlers={{ [SHOP_CTA]: () => setShopHits((hits) => hits + 1) }}
          onError={() => setStatus("render error — see console")}
        />
        {showCta &&
          ctas.map((region) => (
            <div
              key={region.id}
              className="cta-box"
              style={{
                left: `${region.normalizedBounds.x * 100}%`,
                top: `${region.normalizedBounds.y * 100}%`,
                width: `${region.normalizedBounds.width * 100}%`,
                height: `${region.normalizedBounds.height * 100}%`,
              }}
            >
              <span className="cta-tag">{region.id}</span>
            </div>
          ))}
      </div>

      <div className="badge">
        <b>{fixture}</b> — {status}
      </div>
      <div className="badge">
        {ctaSummary}
        {lastCta ? ` · last activation: ${lastCta}` : ""}
        {shopHits > 0 ? ` · shop-cta fired ${shopHits}×` : ""}
      </div>
    </div>
  );
}
