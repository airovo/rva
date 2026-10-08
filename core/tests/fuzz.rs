use rva_core::Asset;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn demo_bytes() -> Vec<u8> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/node/hero.rva");
    std::fs::read(path).expect("read hero.rva")
}

/// Tiny deterministic PRNG so the fuzz corpus is reproducible.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }
}

#[test]
fn parser_survives_mutated_input() {
    let base = demo_bytes();
    let mut rng = Lcg(0x1234_5678_9abc_def0);

    for _ in 0..1500 {
        let mut bytes = base.clone();
        let mutations = 1 + (rng.next() % 8) as usize;
        for _ in 0..mutations {
            let index = (rng.next() as usize) % bytes.len();
            match rng.next() % 3 {
                0 => bytes[index] ^= 0xff,
                1 => bytes[index] = (rng.next() & 0xff) as u8,
                _ => {}
            }
        }
        if rng.next().is_multiple_of(5) {
            let length = (rng.next() as usize) % bytes.len();
            bytes.truncate(length);
        }

        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let _ = Asset::from_bytes(bytes);
        }));
        assert!(outcome.is_ok(), "parser panicked on mutated input");
    }
}

#[test]
fn parser_rejects_absurd_and_tiny_input() {
    let base = demo_bytes();

    // Huge declared manifest/data lengths must error, not panic.
    let mut absurd = base.clone();
    absurd[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
    absurd[24..32].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = Asset::from_bytes(absurd);
    }))
    .is_ok());

    // Empty and truncated inputs.
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = Asset::from_bytes(Vec::new());
    }))
    .is_ok());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = Asset::from_bytes(base[..10].to_vec());
    }))
    .is_ok());
}
