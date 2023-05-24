#![no_std]
#![no_main]

use core::mem::size_of;

use drv0 as _;
use drv1 as _;

use drv_common::CallEntry;

#[no_mangle]
fn main() {
    libos::init();

    libos::println!("\n[ArceOS Tutorial]: B1\n");
    verify();
}

/* Todo: Implement it */
fn traverse_drivers() {
    extern "C" {
        fn init_calls_start();
        fn init_calls_end();
    }
    // Parse range of init_calls by calling C function.
    let range_start = init_calls_start as usize;
    let range_end = init_calls_end as usize;
    display_initcalls_range(range_start, range_end);
    unsafe {
        let entries = core::slice::from_raw_parts(range_start as *const CallEntry, (range_end - range_start) / size_of::<CallEntry>());
        for entry in entries {
            // For each driver, display name & compatible
            let drv = &(entry.init_fn)();
            display_drv_info(drv.name, drv.compatible);
        }
    }
}

fn display_initcalls_range(start: usize, end: usize) {
    libos::println!("init calls range: 0x{:X} ~ 0x{:X}\n", start, end);
}

fn display_drv_info(name: &str, compatible: &str) {
    libos::println!("Found driver '{}': compatible '{}'", name, compatible);
}

fn verify() {
    traverse_drivers();

    libos::println!("\nResult: Okay!");
}
