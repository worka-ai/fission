# fission-shell-mobile

Mobile shell for the Fission UI framework (iOS and Android).

`fission-shell-mobile` will provide the platform integration layer for running Fission applications
on iOS and Android devices. This includes touch input translation, native lifecycle management,
on-screen keyboard handling, and integration with platform-specific rendering surfaces (Metal on
iOS, Vulkan/OpenGL ES on Android).

## Status

**Planned** -- this crate is a placeholder. The module structure exists but no functionality has
been implemented yet. Contributions and design proposals are welcome.

## Planned scope

- iOS shell using `CAMetalLayer` / UIKit lifecycle hooks.
- Android shell using `NativeActivity` / `SurfaceView`.
- Touch and gesture input mapping to Fission `InputEvent` types.
- Integration with `fission-shell` shared abstractions (`Platform`, `VideoBackend`).
- Safe-area insets and display-cutout awareness.

## License

MIT -- see the [Fission repository](https://github.com/niclasburger/fission) for full documentation.
