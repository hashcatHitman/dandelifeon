// SPDX-FileCopyrightText: Copyright © 2025 hashcatHitman
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Types for representing and simulating the modified Game of Life used by the
//! Dandelifeon.

use core::fmt::{self, Display, Formatter};

use rand::Rng;
use rand::distr::{Distribution, StandardUniform};

/// A [`PetriDish`] is a compact representation of the Dandelifeon game board.
///
/// Internally, it is represented by a [`u64`] array of length 25. Each [`u64`]
/// represents a row of [`Cell`]s, where each [`Cell`] takes up 2 bits. Thus,
/// only the least significant 50 bits are used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PetriDish([u64; 25]);

impl PetriDish {
    /// All possible coordinates except for (12, 12).
    pub(crate) const NONCENTER_COORDS: [(u8, u8); 624] = {
        let mut coords: [(u8, u8); 624] = [(0_u8, 0_u8); 624];
        let mut idx: usize = 0;
        let mut x: u8 = 0;
        while x < 25 {
            let mut y: u8 = 0;
            #[expect(
                clippy::indexing_slicing,
                reason = "Constant panics are compile time."
            )]
            while y < 25 {
                if x == 12 && y == 12 {
                    y += 1;
                }
                coords[idx] = (x, y);
                y += 1;
                idx += 1;
            }
            x += 1;
        }

        coords
    };

    /// A constant containing [the setup found by Cobra1117] using a genetic
    /// algorithm in 2016.
    ///
    /// [the setup found by Cobra1117]: https://www.reddit.com/r/botania/comments/5by0jl/optimal_100round_dandelifeon_setup/
    pub(crate) const OPTIMAL_100_ROUND: Self = {
        let mut current_record: Self = Self::new();

        current_record.set_blocked(7, 14);
        current_record.set_blocked(12, 14);
        current_record.set_blocked(17, 14);
        current_record.set_blocked(9, 19);
        current_record.set_blocked(15, 19);
        current_record.set_blocked(17, 20);
        current_record.set_blocked(17, 23);

        current_record.set_living(15, 20);
        current_record.set_living(12, 21);
        current_record.set_living(15, 21);
        current_record.set_living(11, 22);
        current_record.set_living(12, 22);
        current_record.set_living(13, 22);

        current_record
    };

    /// Creates a new [`PetriDish`] where all cells are [`Cell::Dead`] except
    /// for the [`Cell::Dandelifeon`].
    const fn new() -> Self {
        let mut new: Self = Self::new_with([0; 25]);
        new.set_dandelifeon();
        new
    }

    /// Creates a new [`PetriDish`] with the provided array. This performs no
    /// validation of any kind!
    const fn new_with(state: [u64; 25]) -> Self {
        Self(state)
    }

    /// Reads the [`Cell`] at the given coordinates as a [`u8`]. If the
    /// coordinates given are out of bounds, reads a [`Cell::Dead`].
    const fn read(&self, x: u8, y: u8) -> u8 {
        match (x, y) {
            #[expect(
                clippy::indexing_slicing,
                reason = "y is always in the range 0..25 on this branch, it
                cannot panic"
            )]
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "x is always in the range 0..25 on this branch, it
                cannot overflow"
            )]
            (0..25, 0..25) => ((self.0[y as usize] >> (2 * x)) & 0b11) as u8,
            _ => 0,
        }
    }

    /// Reads the [`Cell`] at the given coordinates as a [`u8`], where the x
    /// coordinate may be [`None`], indicating an underflow that would have been
    /// out of bounds. If the coordinates given are out of bounds, reads a
    /// [`Cell::Dead`].
    const fn read_maybe_x(&self, x: Option<u8>, y: u8) -> u8 {
        match (x, y) {
            #[expect(
                clippy::indexing_slicing,
                reason = "y is always in the range 0..25 on this branch, it
                cannot panic"
            )]
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "x is always in the range 0..25 on this branch, it
                cannot overflow"
            )]
            (Some(x @ 0..25), 0..25) => {
                ((self.0[y as usize] >> (2 * x)) & 0b11) as u8
            }
            _ => 0,
        }
    }

    /// Reads the [`Cell`] at the given coordinates as a [`u8`], where the y
    /// coordinate may be [`None`], indicating an underflow that would have been
    /// out of bounds. If the coordinates given are out of bounds, reads a
    /// [`Cell::Dead`].
    const fn read_maybe_y(&self, x: u8, y: Option<u8>) -> u8 {
        match (x, y) {
            #[expect(
                clippy::indexing_slicing,
                reason = "y is always in the range 0..25 on this branch, it
                cannot panic"
            )]
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "x is always in the range 0..25 on this branch, it
                cannot overflow"
            )]
            (0..25, Some(y @ 0..25)) => {
                ((self.0[y as usize] >> (2 * x)) & 0b11) as u8
            }
            _ => 0,
        }
    }

    /// Reads the [`Cell`] at the given coordinates as a [`u8`], where either
    /// coordinate may be [`None`], indicating an underflow that would have been
    /// out of bounds. If the coordinates given are out of bounds, reads a
    /// [`Cell::Dead`].
    const fn read_maybe_x_y(&self, x: Option<u8>, y: Option<u8>) -> u8 {
        match (x, y) {
            #[expect(
                clippy::indexing_slicing,
                reason = "y is always in the range 0..25 on this branch, it
                cannot panic"
            )]
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "x is always in the range 0..25 on this branch, it
                cannot overflow"
            )]
            (Some(x @ 0..25), Some(y @ 0..25)) => {
                ((self.0[y as usize] >> (2 * x)) & 0b11) as u8
            }
            _ => 0,
        }
    }

    /// Writes a value to the [`PetriDish`] at the given coordinates, assuming
    /// that the location being written to is [`Cell::Dead`].
    ///
    /// # Panics
    ///
    /// This method panics if `y >= 25`.
    ///
    /// This method panics if `((value as u64) << (x * 2))` would overflow while
    /// overflow checks are enabled.
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "only used in PetriDish::reduce_cells where x and y are
        guaranteed to be in the range 0..25."
    )]
    const fn write_assume_clean_slate(&mut self, x: u8, y: u8, value: u8) {
        let whole: u64 = self.0[y as usize];
        self.0[y as usize] = whole | ((value as u64) << (x * 2));
    }

    /// Writes a value to the [`PetriDish`] at the given coordinates.
    ///
    /// # Panics
    ///
    /// This method panics if `y >= 25`.
    ///
    /// This method panics if `((value as u64) << (x * 2))` would overflow while
    /// overflow checks are enabled.
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "only used in Hive::explore using coordinates from
        PetriDish::NONCENTER_COORDS, where x and y are guaranteed to be in the
        range 0..25."
    )]
    pub(crate) const fn write(&mut self, x: u8, y: u8, value: u8) {
        self.set_dead(x, y);
        let whole: u64 = self.0[y as usize];
        self.0[y as usize] = whole | ((value as u64) << (x * 2));
    }

    /// Writes a [`Cell::Dead`] to the [`PetriDish`] at the given coordinates.
    ///
    /// # Panics
    ///
    /// This method panics if `y >= 25`.
    ///
    /// This method may panic if `((OLD as u64) << (x * 2))` would overflow
    /// while overflow checks are enabled, where `OLD` is any of `0b00`, `0b01`,
    /// `0b10` or `0b11`.
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "only used in
        <StandardUniform as Distribution<PetriDish>>::sample, where x and y are
        guaranteed to be in the range 0..25 and other functions which meet the
        same criteria."
    )]
    const fn set_dead(&mut self, x: u8, y: u8) {
        let whole: u64 = self.0[y as usize];
        let read: u8 = self.read(x, y);
        self.0[y as usize] = whole ^ ((read as u64) << (2 * x));
    }

    /// Writes a [`Cell::Living`] to the [`PetriDish`] at the given coordinates.
    ///
    /// # Panics
    ///
    /// This method panics if y >= 25.
    ///
    /// This method panics if `((Cell::LIVING as u64) << (x * 2))` would
    /// overflow while overflow checks are enabled.
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "only used in constants and
        <StandardUniform as Distribution<PetriDish>>::sample, where x and y are
        guaranteed to be in the range 0..25."
    )]
    const fn set_living(&mut self, x: u8, y: u8) {
        self.set_dead(x, y);
        let whole: u64 = self.0[y as usize];
        self.0[y as usize] = whole | ((Cell::LIVING as u64) << (2 * x));
    }

    /// Writes a [`Cell::Blocked`] to the [`PetriDish`] at the given
    /// coordinates.
    ///
    /// # Panics
    ///
    /// This method panics if y >= 25.
    ///
    /// This method panics if `((Cell::BLOCKED as u64) << (x * 2))` would
    /// overflow while overflow checks are enabled.
    #[expect(
        clippy::arithmetic_side_effects,
        clippy::indexing_slicing,
        reason = "only used in constants and
        <StandardUniform as Distribution<PetriDish>>::sample, where x and y are
        guaranteed to be in the range 0..25."
    )]
    const fn set_blocked(&mut self, x: u8, y: u8) {
        self.set_dead(x, y);
        let whole: u64 = self.0[y as usize];
        self.0[y as usize] = whole | ((Cell::BLOCKED as u64) << (2 * x));
    }

    /// Writes a [`Cell::Dandelifeon`] to the [`PetriDish`] at (12, 12).
    const fn set_dandelifeon(&mut self) {
        let whole: u64 = self.0[12];
        self.0[12] = whole | ((Cell::DANDELIFEON as u64) << (2 * 12));
    }

    /// Takes a [`Cell`] as a [`u8`] and its neighbors as an array of 8 [u8]s
    /// and determines the state of the cell in the next iteration of the game.
    ///
    /// The transition rules are:
    ///  1) Any live cell with exactly 2 or 3 live neighbours survives the step.
    ///  2) Any live cell not satisfying condition 1 becomes dead.
    ///  3) Any dead cell with exactly three live neighbours becomes a live
    ///     cell.
    fn reduce_local_asm(center: u8, neighbors: [u8; 8]) -> u8 {
        const _: () = const {
            assert!(true as u8 == 1, "true must be 1");
            assert!(false as u8 == 0, "false must be 0");
        };

        let mut count_neighbors: u8 = 0;

        #[expect(
            clippy::arithmetic_side_effects,
            reason = "there are only 8 cells in the neighbors array, this cannot
            overflow"
        )]
        for number in neighbors {
            count_neighbors += u8::from(number == Cell::LIVING);
        }

        let center_alive: bool = center == Cell::LIVING;
        let center_dead: bool = center == Cell::DEAD;
        let dandelifeon: bool = center == Cell::DANDELIFEON;
        let blocked: bool = center == Cell::BLOCKED;
        let three_living_neighbors: bool = count_neighbors == 3;
        let two_living_neighbors: bool = count_neighbors == 2;
        let next_alive: bool = (center_alive
            & (three_living_neighbors | two_living_neighbors))
            | (center_dead & three_living_neighbors);

        let blocked_or_dandelifeon: bool = dandelifeon | blocked;
        let alive_or_dandelifeon: bool = next_alive | dandelifeon;

        (u8::from(blocked_or_dandelifeon) << 1) | u8::from(alive_or_dandelifeon)
    }

    /// Simulates a single step of the game for the entire [`PetriDish`].
    ///
    /// The transition rules are:
    ///  1) Any live cell with exactly 2 or 3 live neighbours survives the step.
    ///  2) Any live cell not satisfying condition 1 becomes dead.
    ///  3) Any dead cell with exactly three live neighbours becomes a live
    ///     cell.
    fn reduce_cells(self) -> (Self, Status) {
        let x_dandelifeon: u8 = 12;
        let y_dandelifeon: u8 = 12;

        let x_dandelifeon_sub_one: Option<u8> = x_dandelifeon.checked_sub(1);
        let x_dandelifeon_add_one: u8 = x_dandelifeon + 1;

        let y_dandelifeon_sub_one: Option<u8> = y_dandelifeon.checked_sub(1);
        let y_dandelifeon_add_one: u8 = y_dandelifeon + 1;

        let dandelifeon_neighbors: [u8; 8] = [
            self.read(x_dandelifeon_add_one, y_dandelifeon),
            self.read_maybe_x(x_dandelifeon_sub_one, y_dandelifeon),
            self.read_maybe_y(x_dandelifeon, y_dandelifeon_sub_one),
            self.read_maybe_y(x_dandelifeon_add_one, y_dandelifeon_sub_one),
            self.read_maybe_x_y(x_dandelifeon_sub_one, y_dandelifeon_sub_one),
            self.read(x_dandelifeon, y_dandelifeon_add_one),
            self.read(x_dandelifeon_add_one, y_dandelifeon_add_one),
            self.read_maybe_x(x_dandelifeon_sub_one, y_dandelifeon_add_one),
        ];

        if dandelifeon_neighbors
            .iter()
            .any(|dangerous_cell: &u8| matches!(*dangerous_cell, Cell::LIVING))
        {
            return (self, Status::AbrubtEnd);
        }

        let mut game_over: Status = Status::Continue;
        let mut next_state: Self = Self::new();

        let mut y: u8 = 0;
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "x and y are always in the range 0..25, they cannot
            overflow (nor can x_p or y_p)"
        )]
        while y < 25 {
            let y_sub_one: Option<u8> = y.checked_sub(1);
            let y_add_one: u8 = y + 1;

            let mut x: u8 = 0;
            while x < 25 {
                let x_sub_one: Option<u8> = x.checked_sub(1);
                let x_add_one: u8 = x + 1;

                let center: u8 = self.read(x, y);
                let neighbors: [u8; 8] = [
                    self.read(x_add_one, y),
                    self.read_maybe_x(x_sub_one, y),
                    self.read_maybe_y(x, y_sub_one),
                    self.read_maybe_y(x_add_one, y_sub_one),
                    self.read_maybe_x_y(x_sub_one, y_sub_one),
                    self.read(x, y_add_one),
                    self.read(x_add_one, y_add_one),
                    self.read_maybe_x(x_sub_one, y_add_one),
                ];

                let new_cell: u8 = Self::reduce_local_asm(center, neighbors);
                if new_cell == Cell::LIVING
                    && matches!(x, 11..14)
                    && matches!(y, 11..14)
                {
                    game_over = Status::NormalEnd;
                }
                next_state.write_assume_clean_slate(x, y, new_cell);

                x += 1;
            }

            y += 1;
        }
        (next_state, game_over)
    }

    /// Counts the number of [`Cell::Living`] and [`Cell::Blocked`] on the
    /// board. If the game has not started, this is equivalent to the initial
    /// investment. The return is (living, blocked).
    pub(crate) fn count_living_and_blocked(self) -> (u16, u16) {
        let mut count_cells: u16 = 0;
        let mut count_blocked: u16 = 0;
        for x in 0..25 {
            for y in 0..25 {
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "at most, any of these fires 25 * 25 = 625 times.
                    this cannot overflow a u16."
                )]
                match self.read(x, y) {
                    Cell::LIVING => {
                        count_cells += 1;
                    }
                    Cell::BLOCKED => {
                        count_blocked += 1;
                    }
                    Cell::DANDELIFEON | Cell::DEAD => (),
                    #[expect(
                        clippy::unreachable,
                        reason = "read returns values masked by 0b11, which we
                        already exhaustively covered"
                    )]
                    _ => unreachable!(
                        "read returns values masked by 0b11, which we already exhaustively covered"
                    ),
                }
            }
        }
        (count_cells, count_blocked)
    }

    /// Runs a simulation of the board for around 101 iterations, or until the
    /// game ends, whichever comes first. Returns the mana generated. If the
    /// game failed to complete, returns 0.
    pub(crate) fn play(&mut self) -> u16 {
        let mut iters: u8 = 0;
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "
            end is Status, which can only represent 0, 1, or 2
            end is guaranteed to be greater than 0 before subtracting 1 from it
            iters is always at least 1 by the time we subtract from it
            1 - 1 = 0, iters - 0 is always OK
            2 - 1 = 1, iters - 1 is always OK
            loop breaks before iters could ever overflow"
        )]
        loop {
            let end: Status;
            (*self, end) = self.reduce_cells();
            iters += 1;

            if end as u8 > 0 {
                let age: u8 = (iters - (end as u8 - 1)).min(100);
                let score: u16 = self.score(age);

                break score;
            }
            if iters > 101 {
                break 0;
            }
        }
    }

    /// Calculates the mana generated by the game ending in the current board
    /// state.
    fn score(self, age: u8) -> u16 {
        let x_dandelifeon: u8 = 12;
        let y_dandelifeon: u8 = 12;

        let x_dandelifeon_sub_one: Option<u8> = x_dandelifeon.checked_sub(1);
        let x_dandelifeon_add_one: u8 = x_dandelifeon + 1;

        let y_dandelifeon_sub_one: Option<u8> = y_dandelifeon.checked_sub(1);
        let y_dandelifeon_add_one: u8 = y_dandelifeon + 1;

        let dandelifeon_neighbors: [u8; 8] = [
            self.read(x_dandelifeon_add_one, y_dandelifeon),
            self.read_maybe_x(x_dandelifeon_sub_one, y_dandelifeon),
            self.read_maybe_y(x_dandelifeon, y_dandelifeon_sub_one),
            self.read_maybe_y(x_dandelifeon_add_one, y_dandelifeon_sub_one),
            self.read_maybe_x_y(x_dandelifeon_sub_one, y_dandelifeon_sub_one),
            self.read(x_dandelifeon, y_dandelifeon_add_one),
            self.read(x_dandelifeon_add_one, y_dandelifeon_add_one),
            self.read_maybe_x(x_dandelifeon_sub_one, y_dandelifeon_add_one),
        ];

        let mana: u16 = dandelifeon_neighbors.iter().fold(
            0,
            |mana: u16, dangerous_cell: &u8| match *dangerous_cell {
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "given the simulation is correct, at most this
                    can be 60 * 100 * 6 = 36000, which doesn't overflow a u16."
                )]
                Cell::LIVING => mana + u16::from(age) * 60,
                _ => mana,
            },
        );

        mana
    }
}

impl Distribution<PetriDish> for StandardUniform {
    /// Generate a random valid [`PetriDish`], using `rng` as the source of
    /// randomness.
    ///
    /// In order to better facilitate the search for useful boards, this will
    /// never block off more than 2 of the [`Cell::Dandelifeon`]s neighbors, as
    /// doing so would reduce the maximum possible return.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PetriDish {
        let mut temp: PetriDish = PetriDish::new();
        let mut core_blockers: u8 = 0;
        for x in 0..25 {
            for y in 0..25 {
                if x != 12 || y != 12 {
                    match (x, y) {
                        (11..14, 11 | 13) | (11 | 13, 11..14) => {
                            match (
                                core_blockers,
                                rng.random_bool(1.0_f64 / 8.0_f64),
                            ) {
                                #[expect(
                                    clippy::arithmetic_side_effects,
                                    reason = "never overflows, incremented at
                                    most twice"
                                )]
                                (..2, true) => {
                                    core_blockers += 1;
                                    temp.set_blocked(x, y);
                                }
                                (2.., _) | (..2, false) => temp.set_dead(x, y),
                            }
                        }
                        _ => match rng.random_range(0_u8..3_u8) {
                            Cell::LIVING => temp.set_living(x, y),
                            Cell::DEAD => temp.set_dead(x, y),
                            Cell::BLOCKED => temp.set_blocked(x, y),
                            #[expect(
                                clippy::unreachable,
                                reason = "random_range must return a value in
                                0..3"
                            )]
                            _ => unreachable!(
                                "random_range must return a value in 0..3"
                            ),
                        },
                    }
                }
            }
        }
        temp
    }
}

impl Display for PetriDish {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in (0..25).rev() {
            for x in 0..25 {
                match (x, y) {
                    (0..11 | 14..25, 0..25) | (0..25, 0..11 | 14..25) => {
                        write!(f, "  ")?;
                        match self.read(x, y) {
                            Cell::LIVING => write!(f, "O  "),
                            Cell::DEAD => write!(f, ".  "),
                            Cell::BLOCKED => write!(f, "X  "),
                            #[expect(
                                clippy::unreachable,
                                reason = "cells not at (12, 12) must be living,
                                dead, or blocked."
                            )]
                            _ => unreachable!(
                                "cells not at (12, 12) must be living, dead, or blocked."
                            ),
                        }?;
                    }
                    (11..14, 11 | 13) | (11 | 13, 11..14) => {
                        write!(f, " {{")?;
                        match self.read(x, y) {
                            Cell::LIVING => write!(f, "O}} "),
                            Cell::DEAD => write!(f, ".}} "),
                            Cell::BLOCKED => write!(f, "X}} "),
                            #[expect(
                                clippy::unreachable,
                                reason = "cells not at (12, 12) must be living,
                                dead, or blocked."
                            )]
                            _ => unreachable!(
                                "cells not at (12, 12) must be living, dead, or blocked."
                            ),
                        }?;
                    }
                    (12, 12) => {
                        write!(f, "<<*>>")?;
                    }
                    #[expect(
                        clippy::unreachable,
                        reason = "x and y are guaranteed to be in the range
                        0..25"
                    )]
                    (25.., _) | (_, 25..) => unreachable!(
                        "x and y are guaranteed to be in the range 0..25"
                    ),
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Default for PetriDish {
    fn default() -> Self {
        Self::new()
    }
}

/// A [`Cell`] represents a single cell in the Dandelifeon cellular automata
/// game.
///
/// Each cell has a conceptual neighborhood, defined as the Moore neighborhood
/// of the cell (the 8 cells surrounding it).
///
/// # Example
///
/// ```rust
/// use dandelifeon::simulation::Cell;
///
/// // If cell 4 is the "current cell", then every other cell shown here is its
/// // neighbor.
/// let sterile_dish: [[(Cell, u8); 3]; 3] = [
///     [(Cell::Dead, 0), (Cell::Dead, 1), (Cell::Dead, 2)],
///     [(Cell::Dead, 3), (Cell::Dead, 4), (Cell::Dead, 5)],
///     [(Cell::Dead, 6), (Cell::Dead, 7), (Cell::Dead, 8)],
/// ];
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Cell {
    /// A [`Cell::Dead`] is a dead cell. It remains dead until it has exactly
    /// three [`Cell::Living`] neighbors, at which point it becomes a
    /// [`Cell::Living`].
    Dead,

    /// A [`Cell::Living`] is a living cell. It remains alive as long as both of
    /// the following are true:
    ///
    /// - it has two or three [`Cell::Living`] neighbors
    /// - no [`Cell::Living`] has occupied the "lethal" zone surrounding the
    ///   [`Cell::Dandelifeon`]
    ///
    /// In theory, it has an age, which starts at 0 if not created by the
    /// simulation. This age increases every step, up to a maximum of 100.
    ///
    /// In practice, this means that any game with no mid-game interference will
    /// contain only cells with an age equal to minimum between the number of
    /// steps elapsed and 100. Thus, we do not store the age with the cells.
    Living,

    /// A [`Cell::Blocked`] is a cell blocked off by the player or environment.
    /// Assuming no mid-game interference, it will always be blocked, and is
    /// treated like a [`Cell::Dead`] for the purposes of counting living
    /// neighbors.
    Blocked,

    /// The [`Cell::Dandelifeon`] is the Dandelifeon itself. There should only
    /// ever be one, located at the very center of the grid.
    ///
    /// The [`Cell::Dandelifeon`]s neighborhood is uninhabitable. If a
    /// [`Cell::Living`] grows into this area, all [`Cell::Living`] immediately
    /// die, becoming [`Cell::Dead`]. This ends the game, and the
    /// [`Cell::Living`] which grew into the uninhabitable region are converted
    /// into mana.
    ///
    /// The amount of mana per consumed [`Cell::Living`] is calculated as
    /// `A * 60`, where `A` is the age of the cell. This means the maximum
    /// possible mana from one game is 36,000 units, as it is not possible to
    /// get more than 6 cells to grow into this area at once.
    Dandelifeon,
}

impl Cell {
    /// A constant containing the value of [`Cell::Dead`] cast to a [`u8`]. For
    /// use in pattern matching while avoiding magic numbers.
    pub(crate) const DEAD: u8 = Self::Dead as u8;

    /// A constant containing the value of [`Cell::Living`] cast to a [`u8`]. For
    /// use in pattern matching while avoiding magic numbers.
    pub(crate) const LIVING: u8 = Self::Living as u8;

    /// A constant containing the value of [`Cell::Blocked`] cast to a [`u8`]. For
    /// use in pattern matching while avoiding magic numbers.
    pub(crate) const BLOCKED: u8 = Self::Blocked as u8;

    /// A constant containing the value of [`Cell::Dandelifeon`] cast to a [`u8`]. For
    /// use in pattern matching while avoiding magic numbers.
    pub(crate) const DANDELIFEON: u8 = Self::Dandelifeon as u8;
}

const _: () = const {
    let mut index: u8 = 0;
    let mut saw_dead: bool = false;
    let mut saw_living: bool = false;
    let mut saw_blocked: bool = false;

    while index < 3 {
        match index {
            Cell::DEAD => saw_dead = true,
            Cell::LIVING => saw_living = true,
            Cell::BLOCKED => saw_blocked = true,
            _ => panic!("Initial constants are wrong!"),
        }
        index += 1;
    }
    assert!(
        saw_dead && saw_living && saw_blocked,
        "Initial constants are wrong!"
    );
};

/// The result of running a step of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
enum Status {
    /// No [`Cell`]s have entered the uninhabited zone. Keep going!
    Continue = 0,
    /// A [`Cell`] has entered the uninhabited zone, or no [`Cell::Living`]
    /// remains. Game end.
    NormalEnd = 1,
    /// A [`Cell`] started in the uninhabited zone. Game end, award no mana.
    AbrubtEnd = 2,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Continue => write!(f, "Continue"),
            Self::NormalEnd => write!(f, "NormalEnd"),
            Self::AbrubtEnd => write!(f, "AbruptEnd"),
        }
    }
}
