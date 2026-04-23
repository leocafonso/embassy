# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Embassy port for the **Renesas RA** family of Cortex-M MCUs. This is **active/incomplete** work across two sibling repositories:

| Repository | Path | Purpose |
|------------|------|---------|
| **embassy-leo** | `../embassy-leo` | Embassy HAL crate (`embassy-ra`) + examples |
| **ra-data** | `../ra-data` | PAC data extraction & `ra-metapac` code generation |

Changes often need to touch both repos. `ra-metapac` is consumed by `embassy-ra` via path dependency `../../ra-data/build/ra-metapac`.

## Build & Flash

Examples use `probe-rs` as the cargo runner. From an example directory:

```bash
cargo run --release
```

| Example | Chip Feature | Target | probe-rs Chip |
|---------|-------------|--------|---------------|
| `examples/ra6e2` | `r7fa6e2bb3cfm` | `thumbv8m.main-none-eabihf` | `R7FA6E2BB` |
| `examples/ra6m4` | `r7fa6m4af3cfb` | `thumbv8m.main-none-eabihf` | `R7FA6M4AF` |
| `examples/ra2e1` | `r7fa2e1a92dfm` | `thumbv8m.base-none-eabi` | `R7FA2E1A9` |

## Hardware Testing Requirement

**Every change must be tested on real hardware.** After any code change (HAL, option bytes, linker script, PAC generation):

1. Run with a 5-second timeout from the relevant example directory:
   ```bash
   timeout 5s cargo run --release; [ $? -eq 124 ] && echo "OK (timeout)" || echo "FAILED"
   ```
2. Confirm the device boots and the defmt log output is correct before the timeout
3. Exit code 124 means timeout (normal — the board was running); any other non-zero exit is a real failure

The current available hardware targets are **RA2E1** and **RA6E2**. When working on RA2 family changes use `examples/ra2e1`; for RA6 use `examples/ra6e2`.

> **Note on probe-rs and ELF:** probe-rs correctly handles ELF files that include OFS (option byte) regions (verified with a Renesas FSP-generated ELF). However, the Rust toolchain currently produces a **flat binary** (not a standard ELF) for these targets — probe-rs does not recognize it as ELF and does not correctly flash the OFS area. This is an open issue. Until resolved, verify OFS values are correct by reading back flash with `probe-rs read` after flashing.

> **OSIS / ID Code area (`0x01010018`) is not flashable by probe-rs on RA2E1.** Do not define a `#[link_section = ".option_setting_osis"]` static in RA2 examples — it creates a PT_LOAD segment at that address and probe-rs fails with "No flash memory contains the entire requested memory range". The hardware default (`0xFFFFFFFF`) means unlocked, so omitting the static is safe for development. The same caution applies to any `OPTION_SETTING_*` region whose address falls outside the chip's code flash range.

Format all crates:
```bash
./fmtall.sh
```

## PAC Generation (ra-data)

```bash
cd ../ra-data
./d gen-all    # SVD→YAML + YAML→ra-metapac (full pipeline)
./d gen        # SVD→YAML only
./d gen-pac    # YAML→ra-metapac only
./d check      # Compile-check all 461 chip definitions
```

Output lands in `../ra-data/build/`.

## Architecture

### Two-Repo Design

**ra-data** is the data pipeline:
```
sources/          # Raw SVD + Rzone files from Renesas
scripts/          # sanitize_svd.py, fix_split_arrays.py
transforms/       # Per-peripheral YAML transforms (GPT, ICU, MSTP, PFS, PORT, SYSC)
data/registers/   # YAML register definitions
build/data/chips/ # Generated per-chip JSON (461 chips)
build/ra-metapac/ # Generated PAC crate consumed by embassy-ra
```

**embassy-ra** is the HAL:
```
embassy-ra/
├── build.rs          # Code generator: memory.x, peripherals, pin mappings, ICU IRQ bindings
├── src/
│   ├── lib.rs        # init(), Config, re-exports
│   ├── gpio.rs       # Flex, Input, Output (embedded-hal 1.0)
│   ├── interrupt.rs  # ICU event model, bind_interrupts! macro
│   ├── mstp.rs       # Module Stop clock gating
│   ├── time_driver.rs # Embassy time driver (AGT timer)
│   └── system/
│       ├── mod.rs          # Clock types, ClockSource, Clocks
│       ├── option_bytes.rs # OFS0/OFS1 (IWDT, WDT, HOCO freq)
│       ├── ra2.rs          # RA2 clock init (SYSC, no PLL)
│       ├── ra4.rs          # RA4 clock init (PLL available)
│       ├── ra6.rs          # RA6 clock init (PLL + PLL2)
│       └── ra8.rs          # RA8 clock init (WIP)
```

### ICU Interrupt Model

RA MCUs use an **Interrupt Controller Unit (ICU)** that maps peripheral events to IEL (interrupt enable level) slots at runtime — unlike STM32's fixed vector table. `build.rs` allocates IRQ slots at compile time from metadata and generates the dispatch handlers. The `bind_interrupts!` macro in `interrupt.rs` wires handlers. The `EMBASSY_RA_EVENTS` environment variable allows overriding event selection.

### build.rs Responsibilities

- Reads chip metadata from `ra-metapac` to generate `peripherals.rs` (singletons + GPIO pin traits)
- Generates `memory.x` linker script from chip memory map
- Generates ICU interrupt bindings from event metadata
- Selects time driver singleton (AGT0/1 or GPT0-13) via cargo features

### Supported Families

| Series | Families | Clock Notes |
|--------|----------|-------------|
| RA2 | E1, E2, E3, L1, L2, A1, A2, T1 | No PLL; HOCO configured via Option Bytes |
| RA4 | E1, E2, M1, M2, M3, T1, W1, L1, C1 | PLL available |
| RA6 | E1, E2, M1, M2, M4, M5, T2, T3 | PLL + PLL2 (up to 240 MHz) |
| RA8 | M1, D1, E1, T1, P1 | WIP; up to 480 MHz |

## Known Incomplete Work

- RA8 PLL configuration is WIP
- SCI UART, I²C, SPI, ADC: feature flags exist but driver code is not implemented
- GPT time driver: feature flags exist but only AGT driver is implemented

## Reference Patterns

When implementing new peripherals or extending the data pipeline, consult these sibling projects:
- `../stm32-data` — mature STM32 PAC data pipeline (same chiptool-based approach)
- `../mspm0-data` — MSPM0 PAC data pipeline (similar YAML intermediary model)
