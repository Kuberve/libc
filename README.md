[![License: LGPL v3](https://img.shields.io/badge/License-LGPL_v3-blue.svg)](LICENSE)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-orange.svg)]()
[![Platform: aarch64](https://img.shields.io/badge/platform-aarch64-lightgrey.svg)]()

# Kuberve libc

The userspace library for **Kuberve** — syscall wrappers, heap allocator,
UART MMIO access, and `print!`/`println!` macros for `no_std` aarch64
programs running in EL0.

> **Pre-alpha.** API is unstable. Breaking changes happen between commits.

## Why LGPL (not AGPL)

The kernel and userspace utilities are AGPLv3, but `libc` is **LGPLv3**.
This is intentional: a proprietary application must be able to link against
`kuberve-libc` without being forced to open its own source code. This is the
standard split used by Linux (GPL kernel + LGPL glibc) and many other systems.

If you link `kuberve-libc` into a proprietary binary, you must:
- Keep `kuberve-libc` itself under LGPLv3 (do not modify it in a way that
  hides its source).
- Provide a way for users to relink with a modified version (dynamic linking
  or object files — the usual LGPL compliance path).

For most applications, this is automatic and harmless. If you need clarification,
see [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md) or contact us.

## What's inside

| Module | Purpose |
|---|---|
| `sys_*` functions | Thin `asm!` wrappers over every syscall: `write`, `read`, `open`, `close`, `spawn`, `kill`, `wait`, `socket`, `tcp_send`, `tcp_recv`, `gui_create_window`, etc. |
| `Message`, `Stat`, `DirEntry` | Re-exports from `kuberve-abi` |
| `BumpAlloc` | Simple bump allocator over `sys_sbrk`, 64 KiB chunks, window `[0x40800000, 0x40A00000)` |
| `uart` | Direct MMIO access to PL011 UART at `0x09000000` (for userspace drivers) |
| `print!` / `println!` | `core::fmt::Write` over `sys_write(fd=1, ...)` |
| `write_payload` | Extract bytes from an IPC `Message{tag=WRITE}` |

## Quick example

```rust
#![no_std]
#![no_main]

extern crate alloc;

use core::arch::global_asm;
use kuberve_libc::{BumpAlloc, sys_exit, sys_sleep};

global_asm!(
    ".section .text._start",
    ".global _start",
    "_start:",
    "  bl user_main",
    "  mov x0, #0",
    "  mov x8, #2",
    "  svc #0",
    "1: b 1b",
);

#[global_allocator]
static GLOBAL: BumpAlloc = BumpAlloc;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { sys_exit(1) }

#[no_mangle]
pub extern "C" fn user_main() -> ! {
    kuberve_libc::println!("hello from EL0");
    loop { sys_sleep(1); }
}
```

## Prerequisites

- Rust nightly
- `aarch64-unknown-none` target

## Build

The library depends on `kuberve-abi` via a `git` dependency (pulled automatically by Cargo).

```bash
git clone https://github.com/Kuberve/libc.git
cd libc
cargo build --release
```

Library output: `target/aarch64-unknown-none/release/libkuberve_libc.rlib`.
Usually not consumed directly — `kuberve-user` depends on this crate via a `git` dependency.

## Related repos (Kuberve Org)

- [kuberve-abi](https://github.com/Kuberve/abi) — syscall numbers, `Message`, shared types (MIT)
- [kuberve-user](https://github.com/Kuberve/user) — userspace programs (AGPLv3)
- [kuberve-kernel](https://github.com/Kuberve/kernel) — the kernel (AGPLv3)
- [kuberve-docs](https://github.com/Kuberve/docs) — architecture, roadmap
- [kuberve-spec](https://github.com/Kuberve/spec) — formal ABI specs

## Contributing

Pull requests are accepted, but may take time to review. Please read
[CONTRIBUTING.md](CONTRIBUTING.md) and sign the [CLA](CLA.md) before opening
a PR.

## License

- **Library code:** LGPL-3.0-or-later. See [LICENSE](LICENSE).
- **Commercial licensing:** available for closed-source use if LGPL does not
  fit your project. See [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md).
- **Trademark:** "Kuberve" is a trademark. See [TRADEMARK.md](TRADEMARK.md).
- **FAQ:** [LICENSE-FAQ.md](LICENSE-FAQ.md).

---

**Kuberve** is a trademark of the project author.
