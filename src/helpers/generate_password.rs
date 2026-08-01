use rand::{Rng, distr::Alphanumeric};

/// Generates a 12-character password mixing letters, digits and symbols.
/// Each character is picked with a uniform random choice between the
/// character classes, so no class dominates the output.
pub fn generate_password() -> String {
    const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{};:,.<>?";
    let mut rng = rand::rng();
    let length: usize = 12;

    let password: Vec<u8> = (0..length)
        .map(|_| {
            let choice = rng.random_range(0..4);
            match choice {
                0 => rng.sample(Alphanumeric) as u8,
                1 => rng.sample(Alphanumeric) as u8,
                2 => SYMBOLS[rng.random_range(0..SYMBOLS.len())],
                _ => rng.random_range(b'A'..=b'Z'),
            }
        })
        .collect();

    String::from_utf8_lossy(&password).to_string()
}
