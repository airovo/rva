File extension: .rva
Provisional media type: application/x-rva
Target registered media type: image/rva
Container: binary, self-contained
Signature: reserved; to be finalized before public RVA 0.1



rva pack --out hero.rva --optimize --quality 85   # lossy, smallest
rva pack --out hero.rva --optimize --lossless      # lossless WebP
rva pack --out hero.rva                            # zlib only (default)