//! Sharing data between a main thread and an interrupt handler safely.
//!
//! This example uses the [libcore](core)-provided [RefCell](core::cell::RefCell) to safely share
//! access to msp430 peripherals between a main thread and interrupt.
//!
//! As with [timer-unsafe] and [timer-oncecell], this example uses the `TIMER0_A1` interrupt to
//! blink LEDs on the [MSP-EXP430G2](http://www.ti.com/tool/MSP-EXP430G2) development kit.
//!
//! ---

#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use core::cell::RefCell;
use msp430::{critical_section as mspcs, interrupt as mspint};
use msp430_rt::entry;
use msp430fr2476::{interrupt, Peripherals};

static PERIPHERALS: mspint::Mutex<RefCell<Option<Peripherals>>> =
    mspint::Mutex::new(RefCell::new(None));

fn init(cs: mspint::CriticalSection) {
    let p = Peripherals::take().unwrap();

    let wdt = &p.WDT_A;

    // Write watchdog password and set hold bit
    wdt.wdtctl.write(unsafe {| w| w
        .wdtpw().bits(0x5a)
        .wdthold().set_bit()
    });

   let p1 = &p.P1;

    // Set P1.0 as output
    p1.p1dir.write(unsafe { |w| w.bits(1 << 0) });
    p1.p1out.write(unsafe { |w| w.bits(1 << 0) });

    // Set P1.0 function 0 P1SEL0 = 0 and P1SEL1 = 0
    p1.p1sel0.write(unsafe { |w| w.bits(0) });
    p1.p1sel1.write(unsafe { |w| w.bits(0) });

    let clock = &p.CS;
    clock.csctl3.modify(unsafe {|_, w| w.bits(1 << 5)});
    clock.csctl1.modify(unsafe {|_, w| w.bits(1 << 0 | 1 << 3 | 3 << 6)});

    let timer = &p.TA3;
    timer.ta3ccr0.write(unsafe {|w| w.bits(1200)});
    timer.ta3ctl.modify(|_, w| w.tassel().bits(1).mc().bits(1)); // tassel().tassel_1().mc().mc_1()
    timer.ta3cctl1.modify(|_, w| w.ccie().set_bit());
    timer.ta3ccr1.write(unsafe {|w| w.bits(600)});

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
fn DefaultHandler(cs: CriticalSection) {
    let p_ref = PERIPHERALS.borrow(cs).borrow();
    let p = p_ref.as_ref().unwrap();

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
