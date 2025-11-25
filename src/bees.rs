// SPDX-FileCopyrightText: Copyright © 2025 hashcatHitman
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! An implementation of the "standard bees algorithm", based on pseudocode like
//! so:
//!
//! ```text
//! Pseudocode for the standard bees algorithm:
//!    1 for i = 1, ..., ns
//!        i  scout[i] = Initialise_scout()
//!        ii flower_patch[i] = Initialise_flower_patch(scout[i])
//!    2 do until stopping_condition = TRUE
//!        i   Recruitment()
//!        ii  for i = 1, ..., na
//!              1 flower_patch[i] = Local_search(flower_patch[i])
//!              2 flower_patch[i] = Site_abandonment(flower_patch[i])
//!              3 flower_patch[i] = Neighbourhood_shrinking(flower_patch[i])
//!        iii for i = nb, ..., ns
//!              1 flower_patch[i] = Global_search(flower_patch[i])}
//! ```

use core::array;
use core::cmp::PartialOrd;
use core::fmt::{Debug, Display};
use core::marker::PhantomData;

use rand::Rng;
use rand::distr::{Distribution, StandardUniform};

/// Implementing [`Colony`] on a struct allows the bees algorithm to be run from
/// it.
///
/// The const generic [usize] `N_SCOUTS` is what is typically referred to as
/// `ns`, and is the number of scout bees.
pub trait Colony<const N_SCOUTS: usize>: Debug + Copy
where
    StandardUniform: Distribution<<Self as Colony<N_SCOUTS>>::Flower>,
{
    /// The [`Colony::Flower`] type represents a possible solution to the
    /// problem being optimized.
    type Flower: Debug + Copy + Display;

    /// The [`Colony::Nectar`] type represents an evaluation of a
    /// [`Colony::Flower`], where smaller is better. [`Ord`] is a hard
    /// requirement. If you want to use a type that is not [`Ord`], such as
    /// [`f64`], you must wrap it in a newtype and manually implement [`Ord`]
    /// according to your needs.
    type Nectar: Debug + Copy + Display + Ord;

    /// Commonly denoted `ne`, this represents the number of elite sites. It
    /// should be smaller than `N_SCOUTS` and [`Colony::BEST_SITES`].
    const ELITE_SITES: usize;

    /// Commonly denoted `nb`, this represents the number of best sites. It
    /// should be smaller than `N_SCOUTS` and larger than
    /// [`Colony::ELITE_SITES`].
    const BEST_SITES: usize;

    /// Commonly denoted `nre`, this represents the number of bees recruited for
    /// elite sites. It should be equal to or greater than
    /// [`Colony::BEST_RECRUITS`].
    const ELITE_RECRUITS: usize;

    /// Commonly denoted `nrb`, this represents the number of bees recruited for
    /// best sites. It should be equal to or less than
    /// [`Colony::ELITE_RECRUITS`].
    const BEST_RECRUITS: usize;

    /// Commonly denoted `a(0)`, this represents the initial size of a flower
    /// patch, or "neighborhood". It should start large typically large enough
    /// to initially consider any possible [`Colony::Flower`] in your solution
    /// space.
    const FLOWER_PATCH_SIZE: usize;

    /// Commonly denoted `stlim`, this represents the limit of stagnation cycles
    /// for site abandonment.
    const STAGNATION_LIMIT: usize;

    /// Due to integer math, there is a lower limit on the "radius" seen at
    /// runtime which may be higher than 0. This constant represents that lower
    /// limit, for whatever use it may be.
    const MINIMUM_RADIUS: usize = {
        let mut size: usize = Self::FLOWER_PATCH_SIZE;
        let mut i: usize = 0;
        while i < Self::STAGNATION_LIMIT {
            size = size - ((size.div_euclid(10)) * 2);
            i += 1;
        }
        size
    };

    /// Evaluate the fitness of a [`Colony::Flower`].
    fn evaluate(solution: &Self::Flower) -> Self::Nectar;

    /// Explore random nearby [`Colony::Flower`]s based on the radius size. The
    /// current best is also provided for consideration.
    fn explore(
        origin: &Self::Flower,
        current_best: &Scout<N_SCOUTS, Self>,
        radius: usize,
    ) -> Self::Flower;

    /// Return `true` when it is time to stop searching. Use the implementor of
    /// this trait to manage needed state (such as iteration counts).
    fn stopping_condition(&mut self) -> bool;

    /// Assumes `flower_patches` is sorted by fitness. Of the `N_SCOUTS`
    /// [`Colony::Flower`]s visited, [`Colony::BEST_SITES`] perform the waggle
    /// dance. Of those scouts, the [`Colony::ELITE_SITES`] very best will
    /// recruit [`Colony::ELITE_RECRUITS`] foragers, and the remaining
    /// [`Colony::BEST_SITES`] - [`Colony::ELITE_SITES`] will recruit
    /// [`Colony::BEST_RECRUITS`].
    fn waggle_dance(
        flower_patches: &mut [FlowerPatch<N_SCOUTS, Self>; N_SCOUTS],
    ) {
        #[expect(
            clippy::needless_range_loop,
            reason = "Codegen is better like this, actually."
        )]
        #[expect(
            clippy::indexing_slicing,
            reason = "index is correct if invariants are upheld"
        )]
        for index in 0..Self::ELITE_SITES {
            flower_patches[index].foragers = Self::ELITE_RECRUITS;
        }

        #[expect(
            clippy::needless_range_loop,
            reason = "Codegen is better like this, actually."
        )]
        #[expect(
            clippy::indexing_slicing,
            reason = "index is correct if invariants are upheld"
        )]
        for index in Self::ELITE_SITES..Self::BEST_SITES {
            flower_patches[index].foragers = Self::BEST_RECRUITS;
        }
    }

    /// The bees algorithm in full. Returns the [`Scout`] with the current best
    /// [`Colony::Flower`].
    fn bees<R: Rng>(&mut self, rng: &mut R) -> Scout<N_SCOUTS, Self>
    where
        StandardUniform: Distribution<Self::Flower>,
    {
        let mut flower_patches: [FlowerPatch<N_SCOUTS, Self>; N_SCOUTS] =
            array::from_fn(|_| FlowerPatch::new(rng));

        flower_patches.sort_by_key(|flower_patch| flower_patch.scout.fitness);

        let mut current_best: Scout<N_SCOUTS, Self> = flower_patches[0].scout;

        while !self.stopping_condition() {
            flower_patches
                .sort_by_key(|flower_patch| flower_patch.scout.fitness);

            if flower_patches[0].scout.fitness < current_best.fitness {
                current_best = flower_patches[0].scout;
                println!(
                    "New best:\n\tFitness:{}\n\tSolution:\n{}",
                    current_best.fitness, current_best.solution
                );
            }
            Self::waggle_dance(&mut flower_patches);

            #[expect(
                clippy::needless_range_loop,
                reason = "Codegen is better like this, actually."
            )]
            #[expect(
                clippy::indexing_slicing,
                reason = "index is correct if invariants are upheld"
            )]
            for i in 0..Self::BEST_SITES {
                flower_patches[i].local_search(&current_best);
                flower_patches[i].abandonment(&mut current_best, rng);
                flower_patches[i].shrinking();
            }

            #[expect(
                clippy::needless_range_loop,
                reason = "Codegen is better like this, actually."
            )]
            #[expect(
                clippy::indexing_slicing,
                reason = "index is correct if invariants are upheld"
            )]
            for i in Self::BEST_SITES..N_SCOUTS {
                flower_patches[i] = FlowerPatch::new(rng);
            }
        }
        current_best
    }
}

/// Each [`Scout`] is a bee who has found a [`Colony::Flower`]. It is evaluated
/// immediately when the [`Scout`] is created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scout<const NS: usize, Hive: Colony<NS>>
where
    StandardUniform: Distribution<Hive::Flower>,
{
    /// The recorded fitness of the held [`Colony::Flower`].
    fitness: Hive::Nectar,
    /// The held [`Colony::Flower`].
    solution: Hive::Flower,
    /// The [`Colony`] this [`Scout`] belongs to.
    _hive: PhantomData<Hive>,
}

impl<const NS: usize, Hive: Colony<NS>> Scout<NS, Hive>
where
    StandardUniform: Distribution<Hive::Flower>,
{
    /// Create a new [`Scout`] assigned to the given [`Colony::Flower`] and
    /// immediately evaluate it.
    pub(crate) fn with_solution(solution: Hive::Flower) -> Self {
        Self {
            fitness: Hive::evaluate(&solution),
            solution,
            _hive: PhantomData,
        }
    }

    /// Get the recorded fitness of the held [`Colony::Flower`].
    pub const fn fitness(&self) -> Hive::Nectar {
        self.fitness
    }

    /// Get the held [`Colony::Flower`].
    pub const fn solution(&self) -> Hive::Flower {
        self.solution
    }
}

/// Each [`FlowerPatch`] describes a "neighbourhood", enabling more exploitative
/// local searching.
#[derive(Debug, Clone, Copy)]
pub struct FlowerPatch<const NS: usize, Hive: Colony<NS>>
where
    StandardUniform: Distribution<Hive::Flower>,
{
    /// The [`Scout`] with the current best [`Colony::Flower`] in this
    /// [`FlowerPatch`].
    scout: Scout<NS, Hive>,
    /// The number of foragers recruited to this [`FlowerPatch`].
    foragers: usize,
    /// The "radius" of this [`FlowerPatch`], which is the "distance" a
    /// [`Colony::Flower`] may differ from the current [`Scout`]s
    /// [`Colony::Flower`].
    neighbourhood: usize,
    /// If this [`FlowerPatch`] found a locl improvement in the current cycle.
    stagnation: bool,
    /// How many cycles this [`FlowerPatch`] has gone without finding a local
    /// improvement.
    stagnation_counter: usize,
}

impl<const NS: usize, Hive: Colony<NS>> FlowerPatch<NS, Hive>
where
    StandardUniform: Distribution<Hive::Flower>,
{
    /// Create a new [`FlowerPatch`] with a randomly assigned [`Scout`].
    pub(crate) fn new<R: Rng>(rng: &mut R) -> Self {
        Self::with_solution(rng.random())
    }

    /// Create a new [`FlowerPatch`] with a [`Scout`] assigned to the given
    /// [`Colony::Flower`].
    pub(crate) fn with_solution(solution: Hive::Flower) -> Self {
        Self {
            scout: Scout::with_solution(solution),
            foragers: 0,
            neighbourhood: Hive::FLOWER_PATCH_SIZE,
            stagnation: true,
            stagnation_counter: 0,
        }
    }

    /// Promotes the given [`Scout`] to be the new defining [`Scout`] for this
    /// [`FlowerPatch`].
    pub(crate) const fn promote(&mut self, new_scout: Scout<NS, Hive>) {
        self.scout = new_scout;
        self.stagnation = false;
    }

    /// Have the foragers explore nearby [`Colony::Flower`]s.
    pub(crate) fn local_search(&mut self, current_best: &Scout<NS, Hive>) {
        self.stagnation = true;
        for _ in 0..self.foragers {
            let new_solution = Hive::explore(
                &self.scout.solution,
                current_best,
                self.neighbourhood,
            );
            let new_scout: Scout<NS, Hive> = Scout::with_solution(new_solution);
            if new_scout.fitness < self.scout.fitness {
                self.promote(new_scout);
            }
        }
    }

    /// Shrink the size of this [`FlowerPatch`] if no local improvement was made
    /// this cycle.
    pub(crate) const fn shrinking(&mut self) {
        if self.stagnation {
            self.neighbourhood = self.neighbourhood.saturating_sub(
                (self.neighbourhood.div_euclid(10)).saturating_mul(2),
            );
        }
    }

    /// If no local improvement was made this cycle, increment the stagnation
    /// counter. If it reaches [`Colony::STAGNATION_LIMIT`], this
    /// [`FlowerPatch`] is abandoned and replaced with a new global search.
    pub(crate) fn abandonment<R: Rng>(
        &mut self,
        current_best: &mut Scout<NS, Hive>,
        rng: &mut R,
    ) {
        if self.stagnation {
            if self.stagnation_counter < Hive::STAGNATION_LIMIT {
                self.stagnation_counter =
                    self.stagnation_counter.saturating_add(1);
            } else {
                if self.scout.fitness < current_best.fitness {
                    *current_best = self.scout;
                }
                *self = Self::new(rng);
            }
        }
    }
}
