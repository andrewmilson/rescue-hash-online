#![feature(test, generic_arg_infer)]

mod felt;
mod rescue;
mod utils;

use felt::PrimeFelt;
use wasm_bindgen::prelude::*;

// an allocator optimized for small code size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
struct WasmRescueXLIX {
    modulus: u128,
    primitive_element: u128,
    capacity: usize,
    state_width: usize,
    rounds: usize,
    digest_size: usize,
    inputs: Vec<u128>,
}

#[wasm_bindgen]
impl WasmRescueXLIX {
    pub fn new(
        modulus: String,
        primitive_element: String,
        capacity: usize,
        state_width: usize,
        rounds: usize,
        digest_size: usize,
    ) -> WasmRescueXLIX {
        WasmRescueXLIX {
            modulus: modulus.parse().unwrap(),
            primitive_element: primitive_element.parse().unwrap(),
            capacity,
            state_width,
            rounds,
            digest_size,
            inputs: Vec::new(),
        }
    }

    pub fn update(&mut self, input: String) {
        self.inputs.push(input.parse().unwrap());
    }

    pub fn finish(&self) -> String {
        use felt::*;

        match self.modulus {
            fp_u128::BaseFelt::MODULUS => {
                let mut hasher = rescue::XLIX::<fp_u128::BaseFelt>::new(
                    self.primitive_element.into(),
                    self.capacity,
                    self.state_width,
                    self.rounds,
                    self.digest_size,
                    128, // TODO: specify security level
                );

                self.inputs
                    .iter()
                    .map(|&input| fp_u128::BaseFelt::new(input))
                    .for_each(|input| hasher.update(input));

                hasher
                    .finish()
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\r\n")
            }
            _ => panic!("Unsupported field"),
        }
    }
}
