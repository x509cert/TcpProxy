use rand::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{Rng, distributions::Uniform};

pub fn fuzz_buffer(buffer: &mut [u8], aggressiveness: u32) -> Result<(),()> {

    let mut rng = rand::thread_rng();

    if (rng.next_u32() % 100) >= aggressiveness {
        return Err(());
    }

    let iterations = if rng.next_u32() % 100 <= 90 {
        1
    } else {
        1 + rng.next_u32() % 5
    };    

    for _ in 0..iterations {

        let which_mutation = rng.gen_range(0..9); 

        // Decide on a mutation type
        match which_mutation {
            0 => {
                // Mutation Type 0: Write random bytes at random positions
                print!("R");
                let index = rng.gen_range(0..buffer.len());
                let random_byte = rng.gen::<u8>();
                buffer[index] = random_byte;
            }

            1 => {
                // Mutation Type 1: Flip bits at random positions
                print!("F");
                let index = rng.gen_range(0..buffer.len());
                let bit_to_flip = 1 << rng.gen_range(0..8); // Select a bit to flip
                buffer[index] ^= bit_to_flip;
            }

            2 => {
                // Mutation Type 2: Insert special characters at random positions
                print!("S");
                let special_chars = b"!@#$%^&*()-=_+[]{}|;:,.<>?/\\";
                let index = rng.gen_range(0..buffer.len());
                let special_char = *special_chars.choose(&mut rng).unwrap();
                buffer[index] = special_char;
            }

            3 => {
                // Mutation Type 3: Data truncation
                //print!("T");
                //let new_size = rng.gen_range(0..buffer.len());
                //buffer.truncate(new_size); TODO
            }

            4 => {
                // Mutation Type 4: Replace a subsection with alphanumeric characters
                print!("P");
                let start = rng.gen_range(0..buffer.len()-1);
                let end = rng.gen_range(start..buffer.len()-1);
                for byte in &mut buffer[start..end] {
                    *byte = rng.sample(Alphanumeric);
                }
            }

            5 => {
                // Mutation Type 5: Insert a random Unicode character
                print!("U");
                let index = rng.gen_range(0..buffer.len());
                let codepoint = rng.gen_range(0x4E00..=0x9FFF);
                let unicode_char = char::from_u32(codepoint).unwrap().to_string();
                let unicode_bytes = unicode_char.as_bytes();
                buffer[index..index + unicode_bytes.len()].copy_from_slice(&unicode_bytes[0..unicode_bytes.len()]);
            }

            6 => {
                // Mutation Type 6: Insert a random emoji
                print!("E");
                let index = rng.gen_range(0..buffer.len());
                // example: emojis from U+1F600 to U+1F64F
                let codepoint = rng.gen_range(0x1F600..=0x1F64F);
                let emoji_char = char::from_u32(codepoint).unwrap().to_string();
                let emoji_bytes = emoji_char.as_bytes();
                buffer[index..index + emoji_bytes.len()].copy_from_slice(&emoji_bytes[0..emoji_bytes.len()]);
            }

            7 => {
                // Mutation Type 7: Insert overlong UTF-8 escapes
                print!("O");
                let index = rng.gen_range(0..buffer.len());
                let base_char = rng.gen_range(0x00..=0x7F) as u8; // choosing a base ASCII character
                
                let overlong = match rng.gen_range(0..3) {
                    0 => vec![
                        // 2-byte overlong encoding
                        0b11000000 | (base_char >> 6),
                        0b10000000 | (base_char & 0b00111111),
                    ],
                    1 => vec![
                        // 3-byte overlong encoding
                        0b11100000, 
                        0b10000000 | (base_char >> 6),
                        0b10000000 | (base_char & 0b00111111),
                    ],
                    2 => vec![
                        // 4-byte overlong encoding
                        0b11110000, 
                        0b10000000 | (base_char >> 6),
                        0b10000000 | (base_char & 0b00111111),
                        0b10000000
                    ],
                    _ => vec![],
                };
                buffer[index..index + overlong.len()].copy_from_slice(&overlong[0..overlong.len()]);
            }

            8 => {
                // Mutation Type 8: Insert Unicode Variadic Selector
                print!("V");
                const MAX_VARIADIC_SIZE:usize = 5;
                let vchars = random_unicode_selector(&mut rng, MAX_VARIADIC_SIZE);
                let mut index = rng.gen_range(0..buffer.len() - MAX_VARIADIC_SIZE);
                for ch in vchars {
                    buffer[index] = ch as u8;
                    index += 1;
                }
            }

            9 => {
                // Mutation Type 9: Set or Rest the top-most bit
                let start = rng.gen_range(0..buffer.len()-1);
                let end = rng.gen_range(start..buffer.len()-1);
                for byte in &mut buffer[start..end] {
                    if rng.gen_bool(0.5) {
                        *byte |= 0xF0;
                    } else {
                        *byte &= 0x7F;
                    }
                }
            }

            _ => {
                // Default: Do nothing
            }
        }
    }

    // new line after each fuzzed packet
    println!();

    Ok(())
}


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
