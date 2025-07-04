//! Basic "hello world" blink demo for the [LP-MSP430FR2476](https://www.ti.com/tool/LP-MSP430FR2476)
//! development kit using a software delay loop- in Rust!
//!
//! Although unnecessary for running the demo, this example also shows the syntax for declaring
//! an interrupt.
//!
//! ---

#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use msp430::asm;
use msp430_rt::{entry, interrupt};

fn delay(n: u16) {
    let mut i = 0;
    loop {
        asm::nop();

        i += 1;

        if i == n {
            break;
        }
    }
}

// P1.0 = green LED
#[entry]
fn main() -> ! {
    let periph = msp430fr247x::Peripherals::take().unwrap();

    let wd = periph.wdt_a;

    // Write watchdog password and set hold bit
    wd.wdtctl()
        .write(unsafe { |w| w.wdtpw().bits(0x5a).wdthold().set_bit() });

    // wd.wdtctl
    //     ().modify(|r, w: &mut msp430fr247x::wdt_a::wdtctl::W| unsafe {
    //         w.bits(((r.bits() & 0xFF) | 0x80) + 0x5a00)
    //     });

    let p1 = periph.p1;

    // Set P1.0 as output
    p1.p1dir().write(unsafe { |w| w.bits(1 << 0) });
    p1.p1out().write(unsafe { |w| w.bits(1 << 0) });

    // Set P1.0 function 0 P1SEL0 = 0 and P1SEL1 = 0
    p1.p1sel0().write(unsafe { |w| w.bits(0) });
    p1.p1sel1().write(unsafe { |w| w.bits(0) });

    let pmm = periph.pmm;

    // Unlock LPM5
    pmm.pm5ctl0().write(|w| w.locklpm5().clear_bit());

    loop {
        delay(10_000);

        // toggle outputs
        p1.p1out()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 0)) });
    }
}

#[interrupt]
fn DefaultHandler() {}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
