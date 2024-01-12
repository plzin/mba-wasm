#!/bin/sh
mkdir local
ln www/* local/
ln pkg/mba_wasm.js pkg/mba_wasm_bg.wasm local
python -m http.server -d local
