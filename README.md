<!--
SPDX-FileCopyrightText: Copyright © 2025 hashcatHitman

SPDX-License-Identifier: Apache-2.0 OR MIT
-->

# dandelifeon

[![unsafe forbidden]][safety dance] [![dependency badge]][deps.rs] [![CI status]][CI workflow] [![CodeQL]][CodeQL workflow]

---

## Bee-ing efficient

### Using swarm intelligence to find optimal solutions in _Botania_

#### Botania

_Botania_ is a tech mod for _Minecraft_ created by Vazkii, with a focus on
"natural magic". The primary resource in the mod is "mana", which is primarily
obtained by means of specially crafted "generating flora". Some are trivial,
such as "Hydroangeas", which consume nearby water and slowly produce mana.
Others are more complicated, such as the "Spectrolus", which must consume wool
in order to generate mana, rotating through the available colors. One such
flower is the "Dandelifeon", a flower with high potential for efficiency and
speed.

The Dandelifeon simulates a modified version of Conway's Game of Life in a 25 by
25 block area centered on itself. Each cell has three possible states:

- Alive, if the cell is occupied by a "Cellular Block".
- Dead, if the cell is not occupied (or, strictly speaking, occupied by the
  "air" block).
- Blocked, if the cell is occupied by any other block.

The Neighbors of a cell are the eight cells surrounding any cell (also known as
a Moore neighborhood). Every step of the simulation, the following transitions
happen to each cell simultaneously:

1) Any live cell with exactly 2 or 3 live neighbors survives the step.

2) Any live cell not satisfying condition 1 becomes dead.

3) Any dead cell with exactly three live neighbors becomes a live cell.

Additionally, the following rules are in place:

- All cells have an age; cells not created by the Dandelifeon (aka, placed by
the player) start at zero. Whenever a cell survives a step, its age increases by
one. Whenever a dead cell becomes a live cell, the age of the new live cell
becomes the age of its oldest neighbor, plus one. Either way, the age is capped
at 100.

- The simulation continues until all cells are dead.

- The 3 by 3 area centered on the Dandelifeon is uninhabitable; if any cell
grows into this area, _all_ cells will immediately die, and those which grew
into this specific forbidden area are converted into mana. The amount of mana
per consumed cell is calculated as $A \times 60$, where $A$ is the age of the
cell.

In order to maximize the utility of the Dandelifeon, then, we would like to find
an initial game state that will produce as much mana as possible for as little
initial resources as possible, while reaching the final state as
soon as possible (that is, a solution that takes 500 steps is less desirable
than one that takes only 100 steps to produce the same reward for the same
initial cost).

#### Swarm Intelligence - Bees algorithm

The "bees algorithm" mimics the food foraging behavior of bee colonies and can
be used for combinatorial optimization and continuous optimization.

#### The Project

The goal is to simulate the behavior of the Dandelifeon and use the bees
algorithm to find optimal initial states, as defined before.

Previous attempts to find optimal solutions to the Dandelifeon have been made in
the past, primarily with genetic algorithms:

- <https://www.reddit.com/r/botania/comments/d4udxo/about_dandelifeon_i_fonud_some_good_starting/>
- <https://www.reddit.com/r/botania/comments/5by0jl/optimal_100round_dandelifeon_setup/>

However, whether or not these solutions are truly optimal for my definition is
not known. It is clear that there is possibility for improvements in that the
first link used excessive starting cellular blocks and was improved upon by a
commenter.

#### See also

- [Botania Repository]
- [Botania Website]
- [Lexica Botania: Hydroangeas]
- [Lexica Botania: Spectrolus]
- [Lexica Botania: Dandelifeon]
- [Wikipedia: Swarm intelligence]
- [Wikipedia: Bees algorithm]

## Getting Started

You'll need to install Rust and its package manager, Cargo, on your system.
Please refer to the official [recommended Rust installation method] for your
system.

You should also have some version of git installed. You can refer to the
[Git documentation] if you need help with that.

Clone the repository and navigate inside it:

```bash
git clone https://github.com/hashcatHitman/dandelifeon.git
cd dandelifeon
```

If you'd like to read the documentation, the recommended way to do so is with:

```bash
cargo doc --document-private-items --open
```

Which will open the documentation in your browser.

To build the project, you can do:

```bash
cargo build --profile release --locked
```

Cargo will download the dependencies and compile the project. It will probably
be located at `./target/release/dandelifeon` or
`./target/release/dandelifeon.exe`, depending on your system.

## MSRV Policy

<!-- Adapted from Arti's MSRV policy -->

Our current Minimum Supported Rust Version (MSRV) is 1.91.1.

We may increase the patch level of the MSRV on any release.

Otherwise, we will not increase MSRV on PATCH releases, though our dependencies
might.

We won't increase MSRV just because we can: we'll only do so when we have a
reason. (We don't guarantee that you'll agree with our reasoning; only that
it will exist.)

[unsafe forbidden]: https://img.shields.io/badge/unsafe-forbidden-success.svg
[safety dance]: https://github.com/rust-secure-code/safety-dance/

[dependency badge]: https://deps.rs/repo/github/hashcatHitman/dandelifeon/status.svg
[deps.rs]: https://deps.rs/repo/github/hashcatHitman/dandelifeon

[CI status]: https://github.com/hashcatHitman/dandelifeon/actions/workflows/ci.yml/badge.svg
[CI workflow]: https://github.com/hashcatHitman/dandelifeon/actions/workflows/ci.yml

[CodeQL]: https://github.com/hashcatHitman/dandelifeon/actions/workflows/github-code-scanning/codeql/badge.svg
[CodeQL workflow]: https://github.com/hashcatHitman/dandelifeon/actions/workflows/github-code-scanning/codeql

[Botania Repository]: https://github.com/VazkiiMods/Botania
[Botania Website]: https://botaniamod.net
[Lexica Botania: Hydroangeas]: https://botaniamod.net/lexicon.html#generating_flowers/hydroangeas
[Lexica Botania: Spectrolus]: https://botaniamod.net/lexicon.html#generating_flowers/spectrolus
[Lexica Botania: Dandelifeon]: https://botaniamod.net/lexicon.html#generating_flowers/dandelifeon
[Wikipedia: Swarm intelligence]: https://en.wikipedia.org/wiki/Swarm_intelligence
[Wikipedia: Bees algorithm]: https://en.wikipedia.org/wiki/Bees_algorithm

[recommended Rust installation method]: https://www.rust-lang.org/tools/install

[Git documentation]: https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
