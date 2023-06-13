# `msp430fr2476-blinky`

This project was initialized using [msp430-quickstart](https://github.com/rust-embedded/msp430-quickstart) and can be run in LP-MSP430FR2476
## Dependencies

- [`mspdebug`](https://github.com/dlbeer/mspdebug)
- [`msp430-gcc-opensource`](https://www.ti.com/tool/MSP430-GCC-OPENSOURCE#downloads)
- [`msp430 FET Drivers`](https://software-dl.ti.com/msp430/msp430_public_sw/mcu/msp430/MSP430_FET_Drivers/latest/index_FDS.html)
- If using WSL [usbipd](https://github.com/dorssel/usbipd-win) allow you to connect USB with your linux subsystem

## Build the project

### Example
``` console
$ cargo build --example blinky
```

### Debug
```console
$ cargo build
```

### Release
```console
$ cargo build --release
```

## Debugging 
Once you have an ELF binary built, flash it to your microcontroller. Use [`mspdebug`](https://github.com/dlbeer/mspdebug) to launch a debug session and `msp430-elf-gdb` with the linked gdb script. For the msp430fr2476 launchpad board this looks like the following:

   In one terminal session
   ```console
   $ sudo mspdebug -v 3300 --fet-force-id MSP430FR2476 -C mspdebug.cfg tilib
   ```

   In another terminal session
   ```console
   $ msp430-elf-gdb -x mspdebug.gdb target/msp430-none-elf/debug/app
   ```

   or simply
   ```console
   $ cargo run
   ```

   or
   ```console
   $ cargo run --release
   ```

   This will flash your Rust code to the microcontroller and open a gdb debugging session to step through it.

   To run the code type 
   ```console
   (gdb) continue
   ```

   A breakpoint (in line 57) can be set with
   ```console
   (gdb) break src/main.rs:57
   ```

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
  at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
