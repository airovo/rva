# Changesets

This folder holds [Changesets](https://github.com/changesets/changesets) — small
Markdown files describing pending releases for the `@airovo/*` packages.

Add one with:

```bash
npm run changeset
```

Commit the generated file with your change. On merge to `main`, the Release
workflow opens a "Version Packages" PR; merging that PR publishes the bumped
packages to npm and creates GitHub releases.
