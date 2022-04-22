use crate::Pin;

pub fn pins_to_bits<const N: usize>(pins: &[Pin; N]) -> [u32; N] {
    pins.map(|pin| pin as u32)
}

pub fn reverse_array<const N: usize, T: Default + Copy>(array: [T; N]) -> [T; N] {
    let mut result: [T; N] = [<T>::default(); N];

    for (out_index, in_index) in (0..N).rev().enumerate() {
        result[out_index] = array[in_index];
    }

    result
}
