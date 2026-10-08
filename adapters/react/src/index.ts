// @airovo/rva-react — React wrapper for the framework-neutral <rva-image> web
// component. This is a thin binding: it registers the element (import
// "@airovo/rva-web") and renders it, so React apps get first-class ergonomics over the
// same WASM core.
//
//   import { RVAImage } from "@airovo/rva-react";
//
//   <RVAImage
//     src="/hero.rva"
//     alt="Hero"
//     onRender={(scene) => setStatus(scene.topology)}
//     onCta={(cta) => track(cta.id)}
//     ctaHandlers={{ "shop-now": () => router.push("/product") }}
//   />
//
// The callback props are sugar over the element's native events (`rva-render`,
// `rva-load`, `rva-error`, `rva-cta`); the `ref` still exposes the raw element
// (including `bindCta`, `bytes` and `srcResolver`) for advanced use.

import {
  createElement,
  forwardRef,
  useCallback,
  useEffect,
  useRef,
  useState,
  type HTMLAttributes,
  type MutableRefObject,
  type ReactNode,
} from "react";
import "@airovo/rva-web";

/* ------------------------------- event types ------------------------------- */

export interface Bounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface CtaRegion {
  id: string;
  visible: boolean;
  bounds: Bounds;
  normalizedBounds: Bounds;
}

/** Emitted on every (re)resolution, e.g. after a resize. */
export interface RvaRenderDetail {
  topology: string;
  width: number;
  height: number;
  viability: number;
  degraded: boolean;
  hidden?: string[];
  ctaRegions?: CtaRegion[];
}

/** Emitted when a CTA region is activated (pointer hit-tested by the element). */
export interface RvaCtaDetail {
  id: string;
  region: CtaRegion;
}

export interface RvaLoadDetail {
  description: string;
  src?: string | null;
}

export interface RvaErrorDetail {
  error: unknown;
  src?: string | null;
}

/* --------------------------------- props ----------------------------------- */

export interface RVAImageProps
  extends Omit<HTMLAttributes<HTMLElement>, "onLoad" | "onError"> {
  /** Asset URL (a .rva file). */
  src: string;
  /** Accessible description. */
  alt?: string;
  children?: ReactNode;
  /** Fired on every (re)resolution with the current composition summary. */
  onRender?: (scene: RvaRenderDetail) => void;
  /** Fired with the asset's resolved CTA regions whenever the composition changes. */
  onCtaRegions?: (regions: CtaRegion[]) => void;
  /** Fired for any CTA region activation. */
  onCta?: (cta: RvaCtaDetail) => void;
  /** Per-region-id callbacks: `{ "shop-now": () => navigate() }`. */
  ctaHandlers?: Record<string, (cta: RvaCtaDetail) => void>;
  /** Fired once the asset bytes are parsed. */
  onLoad?: (detail: RvaLoadDetail) => void;
  /** Fired on load or render failure. */
  onError?: (detail: RvaErrorDetail) => void;
}

/* ------------------------------- component --------------------------------- */

export const RVAImage = forwardRef<HTMLElement, RVAImageProps>(function RVAImage(
  {
    src,
    alt,
    className,
    children,
    onRender,
    onCtaRegions,
    onCta,
    ctaHandlers,
    onLoad,
    onError,
    ...rest
  },
  ref
) {
  const [node, setNode] = useState<HTMLElement | null>(null);

  // Keep the latest callbacks without re-attaching listeners on every render.
  const latest = useRef({ onRender, onCtaRegions, onCta, ctaHandlers, onLoad, onError });
  latest.current = { onRender, onCtaRegions, onCta, ctaHandlers, onLoad, onError };

  // Merge the forwarded ref with our internal node state.
  const setRef = useCallback(
    (value: HTMLElement | null) => {
      if (typeof ref === "function") ref(value);
      else if (ref) (ref as MutableRefObject<HTMLElement | null>).current = value;
      setNode(value);
    },
    [ref]
  );

  useEffect(() => {
    if (!node) return;

    const handleRender = (event: Event) => {
      const detail = (event as CustomEvent<RvaRenderDetail>).detail;
      latest.current.onRender?.(detail);
      latest.current.onCtaRegions?.(detail?.ctaRegions ?? []);
    };
    const handleLoad = (event: Event) =>
      latest.current.onLoad?.((event as CustomEvent<RvaLoadDetail>).detail);
    const handleError = (event: Event) =>
      latest.current.onError?.((event as CustomEvent<RvaErrorDetail>).detail);
    const handleCta = (event: Event) => {
      const detail = (event as CustomEvent<RvaCtaDetail>).detail;
      latest.current.onCta?.(detail);
      latest.current.ctaHandlers?.[detail?.id]?.(detail);
    };

    node.addEventListener("rva-render", handleRender);
    node.addEventListener("rva-load", handleLoad);
    node.addEventListener("rva-error", handleError);
    node.addEventListener("rva-cta", handleCta);
    return () => {
      node.removeEventListener("rva-render", handleRender);
      node.removeEventListener("rva-load", handleLoad);
      node.removeEventListener("rva-error", handleError);
      node.removeEventListener("rva-cta", handleCta);
    };
  }, [node]);

  // React does not map className -> class for custom elements, so translate it
  // explicitly; otherwise the element never receives the consumer's classes
  // (and therefore never picks up responsive CSS such as aspect-ratio).
  const props: Record<string, unknown> = { ref: setRef, src, alt, ...rest };
  if (className) props.class = className;
  return createElement("rva-image", props, children);
});
