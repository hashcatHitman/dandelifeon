// SPDX-FileCopyrightText: Copyright © 2025 hashcatHitman
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! A binary that makes use of the library to search for solutions to the
//! Dandelifeon.

use dandelifeon::Hive;
use dandelifeon::bees::Colony as _;
use rand::SeedableRng as _;
use rand::rngs::SmallRng;

/// Runs the bees algorithm to search for solutions to the Dandelifeon.
fn main() {
    let mut rng: SmallRng = SmallRng::seed_from_u64(42);
    let mut hive: Hive = Hive::new();
    let winning = hive.bees(&mut rng);
    let (best, winner) = (winning.fitness(), winning.solution());
    println!("Finished! Best score was {best}. Solution is:\n{winner}");
}
