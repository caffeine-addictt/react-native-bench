use std::{ffi::c_float, thread};

use sysinfo::System;

uniffi::setup_scaffolding!();

#[derive(Copy, Clone, Debug, PartialEq, uniffi::Record)]
pub struct Metrics {
    cpu: c_float,
}

#[uniffi::export]
pub fn get_cpu() -> Metrics {
    let mut sys = System::new_all();
    thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();

    Metrics {
        cpu: sys.global_cpu_usage(),
    }
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
