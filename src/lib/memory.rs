//! Interact with memory

use core::ptr;

pub unsafe fn read(address: *const u32) -> u32 {
    ptr::read_volatile(address)
}
pub unsafe fn write(address: *mut u32, new: u32) {
    ptr::write_volatile(address, new);
}

pub unsafe fn update(address: *mut u32, updater: &dyn Fn(u32) -> u32) {
    write(address, updater(read(address)));
}

pub unsafe fn read_bits<const N: usize>(address: *const u32, bits: &[u32; N]) -> [bool; N] {
    let current = read(address);

    bits.map(|bit| current & (1 << bit) != 0)
}
pub unsafe fn write_bits<const N: usize>(address: *mut u32, bits: &[u32; N], values: [bool; N]) {
    update(address, &|current| {
        bits.iter().zip(values).fold(current, |result, (bit, set)| {
            if set {
                result | (1 << bit)
            } else {
                result & !(1 << bit)
            }
        })
    })
}

pub unsafe fn set_bits(address: *mut u32, bits: &[u32]) {
    update(address, &|current| {
        bits.iter().fold(current, |result, bit| result | (1 << bit))
    })
}
pub unsafe fn clear_bits(address: *mut u32, bits: &[u32]) {
    update(address, &|current| {
        bits.iter()
            .fold(current, |result, bit| result & !(1 << bit))
    })
}
pub unsafe fn toggle_bits(address: *mut u32, bits: &[u32]) {
    update(address, &|current| {
        bits.iter().fold(current, |result, bit| result ^ (1 << bit))
    })
}
