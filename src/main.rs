#![no_main]
#![no_std]

extern crate panic_msp430; // For now, we only have an infinitely-looping panic handler.

use msp430_rt::entry;

#[allow(unused)]
// Bring interrupt vectors into scope so the linker can see them; enabling the "rt"
// feature of msp430fr2476 transitively enables the "device" feature of msp430-rt.
// This prevents default interrupt vectors from being generated.
use msp430fr2476;

#[entry]
fn main() -> ! {
    let periph = msp430fr2476::Peripherals::take().unwrap();

    let wd = periph.WDT_A;

    // Write watchdog password and set hold bit
    wd.wdtctl.write(unsafe {| w| w
        .wdtpw().bits(0x5a)
        .wdthold().set_bit()
    });

    // Another way to do the same would be
    // wd.wdtctl
    //     .modify(|r, w: &mut msp430fr2476::wdt_a::wdtctl::W| unsafe {
    //         w.bits(((r.bits() & 0xFF) | 0x80) + 0x5a00)
    //     });

    let p1 = periph.P1;

    // Set P1.0 as output
    p1.p1dir.write(unsafe { |w| w.bits(1 << 0) });
    p1.p1out.write(unsafe { |w| w.bits(1 << 0) });

    // Set P1.0 function 0 P1SEL0 = 0 and P1SEL1 = 0
    p1.p1sel0.write(unsafe { |w| w.bits(0) });
    p1.p1sel1.write(unsafe { |w| w.bits(0) });

    let pmm = periph.PMM;

    // Unlock LPM5
    pmm.pm5ctl0.write(|w| w.locklpm5().clear_bit());

    let mut ctl: u32 = 0;
    loop {
        if ctl >= 10000 {
            ctl = 0;
            // Clear P1.0
            p1.p1out.write(unsafe { |w| w.bits(0 << 0) });
        } else if ctl == 5000 {
            // Set P1.0
            p1.p1out.write(unsafe { |w| w.bits(1 << 0) });
        }
        ctl += 1;
    }
}

// The compiler will emit calls to the abort() compiler intrinsic if debug assertions are
// enabled (default for dev profile). MSP430 does not actually have meaningful abort() support
// so for now, we create our own in each application where debug assertions are present.
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!();
}
