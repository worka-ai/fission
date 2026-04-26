# fission-shell-web

Web shell for the Fission UI framework (WebAssembly target).

`fission-shell-web` will provide the platform integration layer for running Fission applications
in the browser via WebAssembly. This includes DOM event translation, `requestAnimationFrame`
scheduling, WebGPU / Canvas 2D rendering surface management, and clipboard / IME interop through
the Web APIs.

## Status

**Planned** -- this crate is a placeholder. The module structure exists but no functionality has
been implemented yet. Contributions and design proposals are welcome.

## Planned scope

- WASM entry point and `requestAnimationFrame` event loop.
- DOM event (pointer, keyboard, wheel, IME) translation to Fission `InputEvent`.
- WebGPU surface integration for the Vello rendering backend.
- Canvas 2D fallback renderer.
- Clipboard and drag-and-drop interop via the Web Clipboard and Drag APIs.
- Integration with `fission-shell` shared abstractions (`Platform`, `VideoBackend`).

## License

MIT -- see the [Fission repository](https://github.com/niclasburger/fission) for full documentation.
