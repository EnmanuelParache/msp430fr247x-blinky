//! Sharing data between a main thread and an interrupt handler safely.
//!
//! This example uses the externally-provided [once_cell][once] to safely share access to msp430
//! peripherals between a main thread and interrupt.
//!
//! The different between [OnceCell][once] and [RefCell][ref] is that setting the data contained
//! inside a [OnceCell][once] can be deferred to run time, but can only be set once. In contrast,
//! the data contained within a [RefCell][ref] can be set multiple times throughout a program, but
//! the contained data must be initialized at compile time. Additionally, [RefCell][ref] will
//! panic if a second thread tries to change its value while the first thread is mutating the
//! variable.
//!
//! The [Periperhals](msp430fr247x::Peripherals) type, and individual peripherals never need
//! to be modified. Therefore, [Periperhals](msp430fr247x::Peripherals) (or a subset of the
//! Periperhals _moved_ to another `struct`, if
//! [building](https://blog.japaric.io/brave-new-io/#freezing-the-clock-configuration)
//! higher-level abstractions) are good candidates to [`Send`](core::marker::Send) to a
//! [OnceCell][once]. [OnceCell][once] in general seems to have better space usage than
//! [RefCell][ref] due to its invariants.
//!
//! As with [timer] and [timer-unsafe], this example uses the `TIMER3_A1` interrupt to
//! blink LEDs on the [LP-MSP430FR2476](http://www.ti.com/tool/LP-MSP430FR2476) development kit.
//!
//! [once]: once_cell::unsync::OnceCell
//! [ref]: core::cell::RefCell
//!
//! ---

#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use msp430::{critical_section as mspcs, interrupt as mspint};
use msp430_rt::entry;
use msp430fr247x::{interrupt, Peripherals};
use once_cell::unsync::OnceCell;

static PERIPHERALS: mspint::Mutex<OnceCell<Peripherals>> = mspint::Mutex::new(OnceCell::new());

fn init(cs: mspint::CriticalSection) {
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

    PERIPHERALS.borrow(cs).set(p).ok().unwrap();
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
fn TIMER3_A1(cs: CriticalSection) {
    let p = PERIPHERALS.borrow(cs).get().unwrap();

    let timer = &p.TA3;
    timer.ta3cctl1.modify(|_, w| w.ccifg().clear_bit());

    let p1 = &p.P1;

    // toggle outputs
    p1.p1out
        .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 0)) });
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
