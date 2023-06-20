//! Sharing data between a main thread and an interrupt handler safely using `unsafe`
//! code blocks in contexts where they can't cause
//! (Undefined Behavior)[https://doc.rust-lang.org/reference/behavior-considered-undefined.html].
//!
//! This example uses the normally `unsafe`
//! [`Peripherals::steal()`][steal] method to safely share access to msp430 peripherals between a
//! main thread and interrupt. All uses of [`steal()`] are commented to explain _why_ its usage
//! is safe in that particular context.
//!
//! As with [timer] and [timer-oncecell], this example uses the `TIMER3_A1` interrupt to blink
//! LEDs on the [LP-MSP430FR2476](http://www.ti.com/tool/LP-MSP430FR2476) development kit.
//!
//! [steal]: msp430fr2476::Peripherals::steal
//!
//! ---

#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use msp430::{critical_section as mspcs, interrupt as mspint};
use msp430_rt::entry;
use msp430fr2476::{interrupt, Peripherals};

fn init(_cs: mspint::CriticalSection) {
    let p = Peripherals::take().unwrap();

    let wdt = &p.WDT_A;

    // Write watchdog password and set hold bit
    wdt.wdtctl
        .write(unsafe { |w| w.wdtpw().bits(0x5a).wdthold().set_bit() });

    let p1 = &p.P1;

    // Set P1.0 as output
    p1.p1dir.write(unsafe { |w| w.bits(1 << 0) });
    p1.p1out.write(unsafe { |w| w.bits(1 << 0) });

    // Set P1.0 function 0 P1SEL0 = 0 and P1SEL1 = 0
    p1.p1sel0.write(unsafe { |w| w.bits(0) });
    p1.p1sel1.write(unsafe { |w| w.bits(0) });

    let clock = &p.CS;
    clock.csctl3.modify(unsafe { |_, w| w.bits(1 << 5) });
    clock
        .csctl1
        .modify(unsafe { |_, w| w.bits(1 << 0 | 1 << 3 | 3 << 6) });

    let timer = &p.TA3;
    timer.ta3ccr0.write(unsafe { |w| w.bits(16000) });
    timer.ta3ctl.modify(|_, w| w.tassel().bits(1).mc().bits(1)); // tassel().tassel_1().mc().mc_1()
    timer.ta3cctl1.modify(|_, w| w.ccie().set_bit());
    timer.ta3ccr1.write(unsafe { |w| w.bits(600) });
}

#[entry(interrupt_enable(pre_interrupt = init))]
fn main() -> ! {
    loop {
        mspcs::with(|_cs| {
            // Do something while interrupts are disabled.
        })
    }
}

#[interrupt]
fn TIMER3_A1(_cs: CriticalSection) {
    // Safe because msp430 disables interrupts on handler entry. Therefore the handler
    // has full control/access to peripherals without data races.
    let p = unsafe { Peripherals::steal() };

    let timer = &p.TA3;
    timer.ta3cctl1.modify(|_, w| w.ccifg().clear_bit());

    let p1 = &p.P1;

    // toggle output
    p1.p1out
        .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 0)) });
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
