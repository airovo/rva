// Convenience source loading for @airovo/rva-node.
//
// The normative primitive stays `open(bytes)` (see adapters/contract.json).
// This layer resolves the common ways an asset is addressed so callers can pass
// a filesystem path, a `file://` URL, an `http(s)://` URL, a `data:` URL, or an
// already-loaded `Uint8Array`.

import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

export type RvaSource = string | URL | Uint8Array;

const SCHEME = /^[a-z][a-z0-9+.-]*:\/\//i;
const FILE_SCHEME = /^file:/i;

function decodeDataUrl(text: string): Uint8Array {
  const comma = text.indexOf(",");
  if (comma < 0) throw new Error("malformed data: URL");
  const meta = text.slice(5, comma);
  const payload = text.slice(comma + 1);
  if (/;base64/i.test(meta)) return new Uint8Array(Buffer.from(payload, "base64"));
  return new Uint8Array(Buffer.from(decodeURIComponent(payload), "binary"));
}

async function readUrl(url: URL): Promise<Uint8Array> {
  switch (url.protocol) {
    case "file:":
      return new Uint8Array(await readFile(fileURLToPath(url)));
    case "http:":
    case "https:": {
      const response = await fetch(url);
      if (!response.ok) throw new Error(`fetch ${url.href}: HTTP ${response.status}`);
      return new Uint8Array(await response.arrayBuffer());
    }
    case "data:":
      return decodeDataUrl(url.href);
    default:
      throw new Error(`unsupported URL scheme '${url.protocol}'`);
  }
}

/**
 * Read `.rva` bytes from a path, `file://` URL, `http(s)://` URL or `data:` URL.
 * `Uint8Array` input is returned as-is.
 */
export async function readSource(input: RvaSource): Promise<Uint8Array> {
  if (input instanceof Uint8Array) return input;
  if (input instanceof URL) return readUrl(input);

  if (input.startsWith("data:")) return decodeDataUrl(input);
  if (FILE_SCHEME.test(input) || SCHEME.test(input)) return readUrl(new URL(input));

  // Anything else is a filesystem path (relative or absolute).
  return new Uint8Array(await readFile(input));
}
