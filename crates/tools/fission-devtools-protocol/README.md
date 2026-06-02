# fission-devtools-protocol

`fission-devtools-protocol` contains the versioned data structures exchanged by
Fission runtime developer tools. It is deliberately schema-only: shells, test
servers, CLIs, and future IDE integrations can share these types without pulling
in a renderer, windowing backend, or application shell.

The protocol covers capability discovery, widget-tree snapshots, Core IR
snapshots, layout and semantics snapshots, frame performance samples, trace
manifests, and inspectable frame bundles.
