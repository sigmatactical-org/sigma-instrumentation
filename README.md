# Sigma Racer Instrumentation

Rust workspace for the **Sigma Racer** motorcycle instrument cluster — reusable UI
library, desktop testbed, and vehicle service.

| Crate | Binary | Role |
|-------|--------|------|
| [`sigma-instrumentation/`](sigma-instrumentation/) | *(library)* | Slint dashboard, gauge geometry, themes, display helpers |
| [`testbed/`](testbed/) | `testbed` | Interactive demo — ride simulation, window nav, component testing |

Production cluster binary **`sigma-dash`** lives in the sibling [`sigma-racer`](../sigma-racer/) repo.

## Quick start

```bash
# Interactive ride simulation (desktop window)
cargo run -p testbed

# 800×480 panel — matches sigma-racer-wingman-imx8mp / sigma-racer-wingman-qemu
cargo virt

# Production binary (idle telemetry — same as embedded target)
cd ../sigma-racer && cargo run --bin sigma-dash
```

### Testbed controls

| Key | Action |
|-----|--------|
| `←` / `→` | Cycle windows |
| `1`–`9` | Jump to window |
| `↑` / `Esc` | Return to ride screen |
| `+` | Restart acceleration run |
| `-` | Force deceleration |

## Workspace layout

```
sigma-instrumentation/   # lib — ui/, gauge, theme, display helpers
testbed/                      # dev — XSR900 ride simulation
../sigma-racer/                 # product — sigma-dash binary, vehicle profile
```

## Display modes

Set `SIGMA_DISPLAY_MODE` to `night` (default), `dusk`, or `day`. See
`sigma-instrumentation/src/theme.rs`.

## Embedded build (Wingman)

The Yocto recipe builds **`sigma-dash`** from the [`sigma-racer`](../sigma-racer/) crate:

```bash
bitbake sigma-racer
```

| Item | Value |
|------|-------|
| Binary | `/usr/bin/sigma-dash` |
| systemd | `cluster-ui.service` |
| Environment | `/etc/sigma-racer-wingman/ui.env` |

Full distribution docs: [`sigma-racer-wingman`](../sigma-racer-wingman/README.md).

## Requirements

- Rust 1.86+ (Yocto meta-rust scarthgap) / 1.85+ for local dev
- Slint 1.13.1 (pinned for Yocto Rust 1.86)

## License

MIT OR Apache-2.0 — see `LICENSE-MIT` and `LICENSE-APACHE`.
