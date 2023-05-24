#![no_std]
#![no_main]

use core::mem::size_of;

use drv0 as _;
use drv1 as _;

use drv_common::CallEntry;

#[no_mangle]
fn main() {
    libos::init();

    libos::println!("\n[ArceOS Tutorial]: B0\n");
    verify();
}

/* Todo: Implement it */
fn traverse_drivers() {
    extern "C" {
        fn initcalls_start() -> usize;
        fn initcalls_end() -> usize;
    }
    // Parse range of init_calls by calling C function.
    unsafe {
        let range_start = initcalls_start();
        let range_end = initcalls_end();
        display_initcalls_range(range_start, range_end);
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
