// @rva/react — React wrapper for the framework-neutral <rva-image> web
// component. This is a thin binding: it registers the element (import
// "@rva/web") and renders it, so React apps get first-class ergonomics over the
// same WASM core.
//
//   import { RVAImage } from "@rva/react";
//   <RVAImage src="/hero.rva" alt="Hero" style={{ width: "100%", aspectRatio: "16/9" }} />

import { createElement, forwardRef, type HTMLAttributes, type ReactNode } from "react";
import "@rva/web";

export interface RVAImageProps extends HTMLAttributes<HTMLElement> {
  /** Asset URL (a .rva file). */
  src: string;
  /** Accessible description. */
  alt?: string;
  children?: ReactNode;
}

export const RVAImage = forwardRef<HTMLElement, RVAImageProps>(function RVAImage(
  { src, alt, className, children, ...rest },
  ref
) {
  // React does not map className -> class for custom elements, so translate it
  // explicitly; otherwise the element never receives the consumer's classes
  // (and therefore never picks up responsive CSS such as aspect-ratio).
  const props: Record<string, unknown> = { ref, src, alt, ...rest };
  if (className) props.class = className;
  return createElement("rva-image", props, children);
});
