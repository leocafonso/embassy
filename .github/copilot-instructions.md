# Embassy-LEO Project

Embassy port for the **Renesas RA** family of Cortex-M MCUs. The project spans two repositories that work together:

| Repository | Path | Purpose |
|------------|------|---------|
| **embassy-leo** | `../embassy-leo` | Embassy HAL crate (`embassy-ra`) + examples |
| **ra-data** | `../ra-data` | PAC data extraction & `ra-metapac` code generation |

## Architecture

### ra-data (PAC Repository)

Data-driven PAC generator. Converts Renesas SVD/Rzone sources → YAML intermediary → `ra-metapac` Rust crate.

```
ra-data/
├── sources/          # Raw SVD + Rzone files from Renesas
├── scripts/          # sanitize_svd.py, fix_split_arrays.py
├── transforms/       # Per-peripheral YAML transforms (GPT, ICU, MSTP, PFS, PORT, SYSC)
├── ra-data-types/    # Shared Rust data models (Chip, Peripheral, Memory, Interrupt)
├── ra-data-gen/      # SVD/Rzone → YAML extractor (chips + registers)
├── ra-metapac-gen/   # YAML → ra-metapac Rust crate generator
├── data/
│   └── registers/    # YAML register definitions (icu, mstp, pfs, port, sysc, timer)
├── build/
│   ├── data/chips/   # Generated per-chip JSON (461 chips)
│   └── ra-metapac/   # Generated PAC crate (peripherals/, chips/, metadata)
└── d                 # Task runner script
```

**Generation pipeline:** `./d gen` (SVD→YAML) → `./d gen-pac` (YAML→ra-metapac) or `./d gen-all` (both).

### embassy-ra (HAL Crate)

Embassy HAL for RA family. Uses `ra-metapac` at build time for code generation of singletons, linker scripts, and ICU interrupt bindings.

```
embassy-ra/
├── build.rs          # Code generator: memory.x, peripherals, pin mappings, ICU IRQ bindings
├── src/
│   ├── lib.rs        # init(), Config, re-exports
│   ├── gpio.rs       # Flex, Input, Output (embedded-hal 1.0)
│   ├── interrupt.rs  # ICU event model, bind_interrupts! macro
│   ├── mstp.rs       # Module Stop clock gating
│   ├── time_driver.rs # Embassy time driver (AGT0 16-bit/32-bit)
│   └── system/
│       ├── mod.rs          # Clock types, ClockSource, Clocks
│       ├── option_bytes.rs # OFS0/OFS1 (IWDT, WDT, HOCO freq)
│       ├── ra2.rs          # RA2 clock init (SYSC, no PLL)
│       ├── ra4.rs          # RA4 clock init (PLL available)
│       ├── ra6.rs          # RA6 clock init (PLL + PLL2)
│       └── ra8.rs          # RA8 clock init (up to 480 MHz, WIP)
```

**Key design:** RA's ICU maps events→IEL slots at runtime (unlike STM32's fixed vectors). `build.rs` allocates IRQ slots at compile time and generates dispatch handlers.

**Dependency:** `embassy-ra` → `ra-metapac` (path: `../../ra-data/build/ra-metapac`).

## Supported Families


| RA2 | RA2E1, RA2E2, RA2E3, RA2L1, RA2L2, RA2A1, RA2A2, RA2T1 | No PLL, HOCO from Option Bytes |
| RA4 | RA4E1, RA4E2, RA4M1, RA4M2, RA4M3, RA4T1, RA4W1, RA4L1, RA4C1 | PLL available |
| RA6 | RA6E1, RA6E2, RA6M1, RA6M2, RA6M4, RA6M5, RA6T2, RA6T3 | PLL + PLL2 |


## non-Supported Families
| Series | Families | Notes |
|--------|----------|-------|
| RA0 | RA0E1, RA0E2, RA0L1 | Entry-level |
| RA8 | RA8M1, RA8D1, RA8T1, RA8E1 | Up to 480 MHz |

## Build & Flash

Examples use `probe-rs` as the cargo runner. From any example directory:

```bash
cargo run --release
```

### Example Targets

| Example | Chip Feature | Target | probe-rs Chip |
|---------|-------------|--------|---------------|
| `examples/ra6e2` | `r7fa6e2bb3cfm` | `thumbv8m.main-none-eabihf` | `R7FA6E2BB` |
| `examples/ra6m4` | `r7fa6m4af3cfb` | `thumbv8m.main-none-eabihf` | `R7FA6M4AF` |
| `examples/ra2e1` | `r7fa2e1a92dfm` | `thumbv8m.base-none-eabi` | `R7FA2E1A9` |

## Testing

### embassy-ra (HAL)
From each example directory:
```bash
cd examples/ra6e2 && cargo run --release
cd examples/ra6m4 && cargo run --release
cd examples/ra2e1 && cargo run --release
```

### ra-data (PAC generation)
From the ra-data repository:
```bash
./d gen-all    # Regenerate data + PAC
```
Verify with:
```bash
./d check      # Compile-checks all 461 chip definitions
```
Output lands in `./build/`.

## Reference Repositories

When implementing new peripherals or extending the data pipeline, consult these sibling projects for patterns:
- `../stm32-data` — STM32 PAC data pipeline (mature, same chiptool-based approach)
- `../mspm0-data` — MSPM0 PAC data pipeline (similar YAML intermediary model)

## Known Issues

- RA8 PLL configuration is WIP
- SCI UART, I²C, SPI, ADC drivers have feature flags but no driver implementation yet
- GPT time driver feature flags exist but only AGT driver is implemented
