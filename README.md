# Sigma Racer Instrumentation

[![CI](https://github.com/sigmatactical-org/sigma-instrumentation/actions/workflows/ci.yml/badge.svg)](https://github.com/sigmatactical-org/sigma-instrumentation/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV](https://img.shields.io/badge/MSRV-1.97.0-blue.svg)](https://www.rust-lang.org)

Rust workspace for the **Sigma Racer** motorcycle instrument cluster — reusable UI
library and desktop testbed.

| Crate | Binary | Role |
|-------|--------|------|
| [`sigma-instrumentation/`](sigma-instrumentation/) | *(library)* | Slint dashboard, gauge geometry, themes, `ClusterTelemetry` binding |
| [`testbed/`](testbed/) | `testbed` | Cluster UI + harness — candump replay, rate, day/dusk/night |

Production cluster binary **`sigma-racer-cluster`** lives in the sibling [`sigma-racer-cluster`](../sigma-racer-cluster/) repo.

## Architecture

```
CAN / IPC / candump  →  decode (sigma-racer-telemetry)  →  ClusterTelemetry  →  apply_telemetry()  →  Slint
```

The UI crate never sees raw CAN. Producers map into [`ClusterTelemetry`](sigma-instrumentation/src/telemetry/message.rs) and call [`apply_telemetry`](sigma-instrumentation/src/telemetry/apply.rs) (or the [`TelemetryPresenter`](sigma-instrumentation/src/telemetry/presenter.rs) trait).

## Quick start

```bash
cargo run -p testbed
```

(`cargo virt` is the same alias.)

### Testbed harness

| Control | Action |
|---------|--------|
| **Browse…** | Pick a `candump -L` log |
| **rate** slider | Replay speed 0.25×–4× |
| **display mode** | Cycle day → dusk → night |
| **Park** | Idle RPM + side stand; freeze replay |
| `←` / `→` | Previous / Next window; on Connectivity/Updates, move focus (edge leaves window) |
| `↑` | Back (list → menu, or home to Systems) |
| `↓` | Select / confirm focused action |
| `+` | Restart current log |
| `-` | Halve replay rate |

Hardware face (product):

```
LEFT              RIGHT
Previous          Back
Next              Select
```

Default feed is the baked sample from `sigma-racer-cluster/testdata/sample-ride.log`.

## Display modes

Set `SIGMA_DISPLAY_MODE` to `day` (default), `dusk`, or `night`. The testbed button cycles day → dusk → night.

## Typography

Cluster UI embeds three faces under `ui/fonts/`:

- **DejaVu Sans** (Book + Bold) — window `default-font-family` (dial numerals, values, gear/speed).
  Both cuts must be embedded: FemtoVG picks the closest available face, so
  without the Bold file every `font-weight ≥ 700` silently renders regular.
- **Operation Napalm** — army stencil for chrome labels / units only (`font-family: "Operation Napalm"`)

Importing Napalm alone made it the FemtoVG fallback and dial digits vanished; keep an explicit default sans.

Font licenses ship alongside the files: DejaVu is Bitstream Vera-licensed
(notice required in distributions — `LICENSE-dejavu-sans.txt`); Operation
Napalm is CC0 public domain by GGBotNet (`LICENSE-operation-napalm.txt`).

## Embedded build (Wingman)

The Yocto recipe builds **`sigma-racer-cluster`** from the [`sigma-racer-cluster`](../sigma-racer-cluster/) crate:

```bash
bitbake sigma-racer-cluster
```

Full distribution docs: [`sigma-racer-wingman`](../sigma-racer-wingman/README.md).

## Requirements

- Rust 1.86+ (Yocto meta-rust scarthgap) / 1.85+ for local dev
- Slint 1.13.1 (pinned for Yocto Rust 1.86)

## Brand & artwork

© Sigma Tactical Group. **All rights reserved.**

The Sigma Tactical Group name, logos, marks, artwork, and visual identity are **proprietary**. They are not covered by this repository's source-code license. See [BRANDING.md](BRANDING.md).

## License

MIT OR Apache-2.0 — see `LICENSE-MIT` and `LICENSE-APACHE`.
