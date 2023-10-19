use rand::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{Rng, distributions::Uniform};




// the next two functions create special Unicode characters
fn random_unicode_character<R: Rng>(rng: &mut R) -> char {
    loop {
        let range = Uniform::from(0x0000..=0xFFFF);
        let value = rng.sample(range);

        if char::from_u32(value).is_some() {
            return char::from_u32(value).unwrap();
        }
    }
}

fn random_unicode_selector<R: Rng>(rng: &mut R, count: usize) -> Vec<char> {
    (0..count).map(|_| random_unicode_character(rng)).collect()
}
