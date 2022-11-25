//! https://eprint.iacr.org/2020/1143.pdf

use crate::felt;
use crate::felt::Felt;
use crate::felt::PrimeFelt;
use sha3_const::Shake256;

/// Computes the Reduced Row Echelon Form
///
/// Computed by Gauss–Jordan elimination
/// https://en.wikipedia.org/wiki/Row_echelon_form
fn echelon_form<T: Felt>(m: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    let rows = m.len();

    if rows == 0 {
        return Vec::new();
    }

    let cols = m[0].len();

    if cols == 0 {
        return Vec::new();
    }

    let mut lead = 0;
    let mut m = m.clone();

    for r in 0..rows {
        if cols <= lead {
            return m;
        }

        // Find the pivot
        let mut i = r;
        while m[i][lead].is_zero() {
            i += 1;
            if i == rows {
                i = r;
                lead += 1;

                if lead == cols {
                    return m;
                }
            }
        }

        if i != r {
            m.swap(i, r);
        }

        let pivot = m[r][lead];
        m[r].iter_mut().for_each(|v| *v /= pivot);

        for i in 0..rows {
            if i != r {
                let pivot = m[i][lead];
                for j in lead..cols {
                    m[i][j] = pivot * m[r][j];
                }
            }
        }

        lead += 1;
    }

    m
}

fn transpose<T: Felt>(m: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    let rows = m.len();

    if rows == 0 {
        return Vec::new();
    }

    let cols = m[0].len();

    if cols == 0 {
        return Vec::new();
    }

    let mut m_trans = vec![vec![T::zero(); rows]; cols];

    for (r, row) in m.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            m_trans[c][r] = *val;
        }
    }

    m_trans
}

/// Algorithm 4: https://eprint.iacr.org/2020/1143.pdf
fn get_mds_matrix<E: Felt>(generator: E, m: usize) -> Vec<Vec<E>> {
    // get a systematic generator matrix for the code
    let rows = m;
    let cols = 2 * m;
    let mut V = vec![vec![E::zero(); cols]; rows];
    for i in 0..rows {
        for j in 0..cols {
            V[i][j] = generator.pow(i as u128 * j as u128);
        }
    }
    let mut V = echelon_form(&V);

    // the MDS matrix is the transpose of the right half of this matrix
    V.iter_mut().for_each(|row| *row = row.split_off(cols / 2));
    transpose(&V)
}

fn matrix_mul<E: PrimeFelt>(A: &Vec<Vec<E>>, B: &Vec<Vec<E>>) -> Vec<Vec<E>> {
    if A.len() == 0 || A[0].len() == 0 || B.len() == 0 || B[0].len() == 0 {
        return Vec::new();
    }

    debug_assert_eq!(A[0].len(), B.len());
    let mut res = vec![vec![E::zero(); B[0].len()]; A.len()];

    // iterate rows of A
    for i in 0..A.len() {
        // iterate columns of B
        for j in 0..B[0].len() {
            // iterate rows of B
            for k in 0..B.len() {
                res[i][j] += A[i][k] * B[k][j];
            }
        }
    }

    res
}

fn round_constants<E: PrimeFelt>(
    m: usize,
    capacity: usize,
    security_level: usize,
    N: usize,
) -> Vec<E> {
    // generate pseudorandom bytes
    let bytes_per_int = (E::BITS as f32 / 8f32).ceil() as usize + 1;
    let num_bytes = bytes_per_int * 2 * m * N;
    let seed_string = format!(
        "Rescue-XLIX({},{},{},{})",
        E::MODULUS,
        m,
        capacity,
        security_level
    );
    let mut hasher = Shake256::default();
    hasher.update(seed_string.as_bytes());
    let mut reader = hasher.finalize_xof();

    // process byte string in chunks
    let mut round_constants = Vec::new();
    for i in 0..2 * m * N {
        let mut chunk = vec![0u8; bytes_per_int];
        reader.read(&mut chunk);

        let mut power = E::one();
        let mut acc = E::zero();
        for byte in chunk {
            acc += power * E::from(byte);
            power *= E::from(1u32 << 8);
        }

        round_constants.push(acc);
    }

    round_constants
}

#[test]
fn test_shake() {
    let mut hasher = Shake256::default();
    hasher.update(b"Rescue-XLIX");
    let mut reader = hasher.finalize_xof();
    let mut chunk = vec![0u8; 10];
    reader.read(&mut chunk);
    println!("{:?}", chunk);
}

#[test]
fn test_round_constants() {
    let result = round_constants::<felt::fp_u128::BaseFelt>(2, 1, 128, 27)
        .into_iter()
        .map(|v| v.as_integer())
        .collect::<Vec<_>>();
    println!("{:?} {}", result, result.len());
}

pub struct XLIX<E> {
    generator: E,
    c_p: usize,
    m: usize,
    N: usize,
    round_constants: Vec<E>,
    digest_size: usize,
    MDS: Vec<Vec<E>>,
    input: Vec<E>,
}

impl<E: PrimeFelt> XLIX<E> {
    pub fn new(
        generator: E,
        capacity: usize,
        state_width: usize,
        rounds: usize,
        digest_size: usize,
        security_level: usize,
    ) -> XLIX<E> {
        XLIX {
            generator,
            c_p: capacity,
            m: state_width,
            N: rounds,
            digest_size,
            round_constants: round_constants(state_width, capacity, security_level, rounds),
            MDS: get_mds_matrix(generator, state_width),
            input: Vec::new(),
        }
    }

    pub fn update(&mut self, input: E) {
        self.input.push(input)
    }

    pub fn permute(&self, state: &mut Vec<Vec<E>>) {
        let alpha = 138u128;
        let alpha_inv = E::from(alpha).inverse().unwrap().as_integer();

        for i in 0..self.N {
            // S-box
            for j in 0..self.m {
                state[j][0] = state[j][0].pow(alpha);
            }

            // MDS
            *state = matrix_mul(&self.MDS, &state);

            // constants
            for j in 0..self.m {
                state[j][0] += self.round_constants[i * 2 * self.m + j];
            }

            // inverse S-box
            for j in 0..self.m {
                state[j][0] = state[j][0].pow(alpha_inv);
            }

            // MDS
            *state = matrix_mul(&self.MDS, &state);

            // constants
            for j in 0..self.m {
                state[j][0] += self.round_constants[i * 2 * self.m + self.m + j];
            }
        }
    }

    pub fn finish(&self) -> Vec<E> {
        let rate = self.m - self.c_p;
        let mut input = self.input.clone();

        // apply padding
        input.push(E::one());
        while input.len() % rate != 0 {
            input.push(E::zero());
        }

        // initialize state to all zeros
        let mut state = vec![vec![E::zero(); 1]; self.m];

        // absorbing
        let mut absorb_index = 0;
        while absorb_index < input.len() {
            for i in 0..rate {
                state[i][0] += input[absorb_index];
                absorb_index += 1;
            }
            self.permute(&mut state);
        }

        // squeezing
        let mut output_sequence = Vec::new();
        for i in 0..usize::min(rate, self.digest_size) {
            output_sequence.push(state[i][0]);
        }

        output_sequence
    }
}
