import { useEffect, useRef, useState } from "react";
import { RVAImage } from "@rva/react";

interface RenderDetail {
  topology: string;
  width: number;
  height: number;
  viability: number;
  degraded: boolean;
  hidden?: string[];
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

export function App() {
  const ref = useRef<HTMLElement>(null);
  const [fixture, setFixture] = useState(FIXTURES[0]);
  const [status, setStatus] = useState("resolving…");

  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    const onRender = (event: Event) => {
      const detail = (event as CustomEvent<RenderDetail>).detail;
      const hidden = detail.hidden?.length ? ` · hidden ${detail.hidden.length}` : "";
      setStatus(
        `topology ${detail.topology} · ${detail.width}×${detail.height} · ` +
          `viability ${detail.viability.toFixed(2)}` +
          `${detail.degraded ? " · degraded" : ""}${hidden}`
      );
    };
    const onError = () => setStatus("render error — see console");
    element.addEventListener("rva-render", onRender);
    element.addEventListener("rva-error", onError);
    return () => {
      element.removeEventListener("rva-render", onRender);
      element.removeEventListener("rva-error", onError);
    };
  }, []);

  const pick = (id: string) => {
    if (id === fixture) return;
    setStatus("resolving…");
    setFixture(id);
  };

  return (
    <div className="wrap">
      <h1 style={{ fontSize: 18, margin: 0 }}>RVA — Torture Corpus (React)</h1>
      <p>
        All 15 adversarial fixtures. Pick one, then resize the window to explore
        topology selection, degradation and fallback.
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

      <RVAImage
        ref={ref}
        src={`/fixtures/${fixture}.rva`}
        alt={`RVA fixture ${fixture}`}
        className="hero"
      />
      <div className="badge">
        <b>{fixture}</b> — {status}
      </div>
    </div>
  );
}
