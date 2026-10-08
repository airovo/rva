# RVA Prototype Implementation Notes

The package intentionally separates **asset semantics** from **rendering**.

The first renderer can be simple:
- aspect-ratio topology selection
- normalized coordinates
- raster/vector/text compositing

Then evolve toward:
- hard/soft constraints
- focal-region protection
- min/preferred/max scaling
- optional-element degradation
- topology viability scoring
- deterministic conflict resolution

The `.rva` package can initially be represented as this directory. Once the model is stable, package the same structure into a single container file.
