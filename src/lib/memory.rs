//! Interact with memory

use core::ptr;

use crate::{Bit, L};

pub unsafe fn read(address: *mut u32) -> u32 {
    ptr::read_volatile(address)
}
pub unsafe fn write(address: *mut u32, new: u32) {
    ptr::write_volatile(address, new);
}

pub unsafe fn update<Updater: Fn(u32) -> u32>(address: *mut u32, updater: Updater) {
    write(address, updater(read(address)));
}

pub unsafe fn read_bits<const N: usize>(address: *mut u32, bits: &[u32; N]) -> [bool; N] {
    let current = read(address);
    let mut result = [L; N];

    // TODO: look up accumulate or reduce or something
    for (i, bit) in bits.iter().enumerate() {
        result[i] = (current & (1 << bit)) != 0;
    }

    result
}
pub unsafe fn write_bits<const N: usize>(address: *mut u32, bits: &[u32; N], values: [bool; N]) {
    update(address, |current| {
        let mut new = current;
        // TODO: look up accumulate or reduce or something
        for (bit, set) in bits.iter().zip(values) {
            if set {
                new |= (1 << bit);
            } else {
                new &= !(1 << bit);
            }
        }
        new
    })
}

pub unsafe fn update_bits<const N: usize, Updater: Fn([bool; N]) -> [bool; N]>(address: *mut u32, bits: &[Bit; N], updater: Updater) {
    write_bits(address, bits, updater(read_bits(address, bits)))
}

pub unsafe fn set_bits(address: *mut u32, bits: &[Bit]) {
    update(address, |current| {
        let mut new = current;

        // TODO: look up accumulate or reduce or something
        for bit in bits {
            new |= (1 << bit);
        }

        new
    })
}
pub unsafe fn clear_bits(address: *mut u32, bits: &[Bit]) {
    update(address, |current| {
        let mut new = current;

        // TODO: look up accumulate or reduce or something
        for bit in bits {
            new &= !(1 << bit);
        }

        new
    })
}
pub unsafe fn toggle_bits(address: *mut u32, bits: &[Bit]) {
    update(address, |current| {
        let mut new = current;

        // TODO: look up accumulate or reduce or something
        for bit in bits {
            new ^= (1 << bit);
        }

        new
    })
}
