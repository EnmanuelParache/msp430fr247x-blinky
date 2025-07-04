//! Sharing data between a main thread and an interrupt handler safely.
//!
//! This example uses the [libcore](core)-provided [RefCell](core::cell::RefCell) to safely share
//! access to msp430 peripherals between a main thread and interrupt.
//!
//! This example uses the `PORT4` and `PORT2` interrupts to toggle LED1 on the
//! [LP-MSP430FR2476](http://www.ti.com/tool/LP-MSP430FR2476) development kit.
//! NOTE: No debounce logic is implemnted.
//! ---

#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use core::cell::RefCell;
use msp430::{critical_section as mspcs, interrupt as mspint};
use msp430_rt::entry;
use msp430fr247x::{interrupt, Peripherals};

static PERIPHERALS: mspint::Mutex<RefCell<Option<Peripherals>>> =
    mspint::Mutex::new(RefCell::new(None));

fn init(cs: mspint::CriticalSection) {
    let p = Peripherals::take().unwrap();

    let wdt = &p.wdt_a;

    // Write watchdog password and set hold bit
    wdt.wdtctl()
        .write(unsafe { |w| w.wdtpw().bits(0x5a).wdthold().set_bit() });

    let p1 = &p.p1;

    // Set P1.0 as output
    p1.p1dir().write(unsafe { |w| w.bits(1 << 0) });
    p1.p1out().write(unsafe { |w| w.bits(1 << 0) });

    // Set P1.0 function 0 P1SEL0 = 0 and P1SEL1 = 0
    p1.p1sel0().write(unsafe { |w| w.bits(0) });
    p1.p1sel1().write(unsafe { |w| w.bits(0) });

    // Tunrn LED1 (P1.0) ON
    p1.p1out()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 0)) });

    let p4 = &p.p4;

    p4.p4ifg().write(unsafe { |w| w.bits(0) });

    // PxDIR PxREN PxOUT I/O Configuration
    //     0     0     x Input
    //     0     1     0 Input with pulldown resistor
    //     0     1     1 Input with pullup resistor
    //     1     x     x Output

    // Set P4.0 as input
    p4.p4dir()
        .modify(unsafe { |r, w| w.bits(r.bits() & !(1 << 0)) });
    p4.p4ren()
        .modify(unsafe { |r, w| w.bits(r.bits() | (1 << 0)) });
    p4.p4out()
        .modify(unsafe { |r, w| w.bits(r.bits() | (1 << 0)) });

    // Set P4.0 function 0 P4SEL0 = 0 and P4SEL1 = 0
    p4.p4sel0()
        .modify(unsafe { |r, w| w.bits(r.bits() & !(1 << 0)) });
    p4.p4sel1()
        .modify(unsafe { |r, w| w.bits(r.bits() & !(1 << 0)) });

    // Set low to high transition
    p4.p4ies()
        .modify(unsafe { |r, w| w.bits(r.bits() | (1 << 0)) });

    // Enable interrupt for P4.0
    p4.p4ie()
        .modify(unsafe { |r, w| w.bits(r.bits() | (1 << 0)) });

    let p2 = &p.p2;

    // Set P2.3 as input
    p2.p2dir()
        .modify(unsafe { |r, w| w.bits(r.bits() & !(1 << 3)) });
    p2.p2ren()
        .modify(unsafe { |r, w| w.bits(r.bits() | (1 << 3)) });
    p2.p2out()
        .modify(unsafe { |r, w| w.bits(r.bits() | (1 << 3)) });

    // Set P2.3 function 0 P2SEL0 = 0 and P2SEL1 = 0
    p2.p2sel0()
        .modify(unsafe { |r, w| w.bits(r.bits() & !(1 << 3)) });
    p2.p2sel1()
        .modify(unsafe { |r, w| w.bits(r.bits() & !(1 << 3)) });

    // Set low to high transition
    p2.p2ies()
        .modify(unsafe { |r, w| w.bits(r.bits() | (1 << 3)) });

    // Enable interrupt for P2.3
    p2.p2ie()
        .modify(unsafe { |r, w| w.bits(r.bits() | (1 << 3)) });

    // Power Management Module
    let pmm = &p.pmm;

    // Unlock LPM5
    pmm.pm5ctl0().write(|w| w.locklpm5().clear_bit());

    *PERIPHERALS.borrow(cs).borrow_mut() = Some(p);
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
fn PORT4(cs: CriticalSection) {
    let p_ref = PERIPHERALS.borrow(cs).borrow();
    let p = p_ref.as_ref().unwrap();

    let p4 = &p.p4;

    p4.p4ifg().write(unsafe { |w| w.bits(0) });

    let p1 = &p.p1;

    // toggle outputs
    p1.p1out()
        .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 0)) });
}

#[interrupt]
fn PORT2(cs: CriticalSection) {
    let p_ref = PERIPHERALS.borrow(cs).borrow();
    let p = p_ref.as_ref().unwrap();

    let p2 = &p.p2;

    p2.p2ifg().reset();

    let p1 = &p.p1;

    // toggle outputs
    p1.p1out()
        .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 0)) });
}

#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
