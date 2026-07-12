# Icons

Tauri needs generated icon binaries (`.ico`, `.png`) before it can run or
bundle. They are **not** committed — generate them once from the source logo:

```bash
# from the project root, after `npm install`
npm run tauri icon src-tauri/icons/icon.svg
```

This produces `icon.ico`, `32x32.png`, `128x128.png`, `128x128@2x.png`, etc.
in this folder, which `tauri.conf.json` references.

If your Tauri CLI version rejects SVG input, export `icon.svg` to a
1024×1024 PNG first and pass that PNG instead.
