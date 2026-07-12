# Fonts — LINE Seed Sans TH

The widget uses **LINE Seed Sans TH**. It loads in this order:

1. the font **installed on your system** (`local(...)`) — most Thai / LINE
   users already have it, so nothing more is needed;
2. otherwise the bundled files below.

## To bundle it (optional, for machines without the font installed)

Download LINE Seed (free, SIL Open Font License) from the official site:
<https://seed.line.me/index_en.html> → drop these two files here:

```
public/fonts/LINESeedSansTH_W_Rg.woff2   (Regular / 400)
public/fonts/LINESeedSansTH_W_Bd.woff2   (Bold / 700)
```

If your download only has `.ttf`/`.otf`, convert to `.woff2` (e.g. with
`fonttools` or an online converter) and keep the exact filenames above — they
are referenced by `@font-face` in `src/styles.css`.

Vite serves everything in `public/` at the site root, so these resolve to
`/fonts/...` at runtime.

> License: LINE Seed is distributed under the SIL Open Font License 1.1.
