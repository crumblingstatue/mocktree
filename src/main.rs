use {
    rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng},
    std::{env, error::Error, fs, path::Path},
};

fn wordlist() -> Vec<&'static str> {
    include_str!("../wordlist.txt").lines().collect()
}

struct Dict {
    wordlist: Vec<&'static str>,
    rng: ThreadRng,
}

impl Default for Dict {
    fn default() -> Self {
        Self {
            wordlist: wordlist(),
            rng: thread_rng(),
        }
    }
}

impl Dict {
    fn random_phrase(&mut self) -> String {
        let len = self.rng.gen_range(1..=3);
        self.wordlist
            .choose_multiple(&mut self.rng, len)
            .copied()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// WARNING: These are pretty sensible limits. Going over them may result in filling your
// filesystem to the brim with garbage

/// Min items per folder
const MIN_ITEMS: u8 = 3;
/// Max items per folder
const MAX_ITEMS: u8 = 50;
/// Depth hard limit
const DEPTH_HARD_LIMIT: u8 = 7;
/// Directory hard limit (per directory)
const DIR_HARD_LIMIT: u8 = 4;

/// Creates a directory at `base` and fills it with mock items
fn write_mock_tree(
    dict: &mut Dict,
    base: &Path,
    depth: u8,
    rng: &mut impl Rng,
) -> Result<(), Box<dyn Error>> {
    // Create a directory
    std::fs::create_dir(base)?;
    let mut dirs = 0;
    // Fill it with mock items
    for _ in 0..rng.gen_range(MIN_ITEMS..=MAX_ITEMS) {
        // Determine whether the item should be a directory
        let is_dir = if depth == 0 {
            true
        } else if depth >= DEPTH_HARD_LIMIT {
            false
        } else {
            rng.gen_bool(1.0 / (depth as f64 * 3.0))
        };
        // Calculate item path
        let node_name = dict.random_phrase();
        let path = base.join(node_name);
        if is_dir && dirs < DIR_HARD_LIMIT {
            // If it's a directory, we call recursively with increased depth
            let result = write_mock_tree(dict, &path, depth + 1, rng);
            if let Err(e) = result {
                eprintln!("Error creating mock tree at '{path:?}': {e}");
            }
            dirs += 1;
        } else {
            // Else we just create a plain old file
            let result = fs::File::create(&path);
            if let Err(e) = result {
                eprintln!("Error creating file '{path:?}': {e}");
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let root_path = args.next().expect("Needs root path");
    let mut dict = Dict::default();
    write_mock_tree(&mut dict, root_path.as_ref(), 0, &mut thread_rng())?;
    Ok(())
}
