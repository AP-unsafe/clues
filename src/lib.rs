use std::fmt::Display;

use crate::Bound::{AtLeast, AtMost};

use geometry::{Cell, Map};

// TODO: logs
// TODO: run pedantic clippy

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Clue {
    // No further than ‘radius‘ from ‘centre‘.
    Within { centre: Cell, radius: u32 },
    // Further than ‘radius‘ from ‘centre‘.
    Beyond { centre: Cell, radius: u32 },
    // The coordinate on ‘axis‘ satisfies ‘bound‘.
    // The clue is atleast or atmost n on that axis
    HalfPlane { axis: Axis, bound: Bound },
    // Strictly closer to ‘a‘ than to ‘b‘.
    Nearer { a: Cell, b: Cell },
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Axis {
    Q,
    R,
    S,
}

impl Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Axis::Q => "Q",
            Axis::R => "R",
            Axis::S => "S",
        };

        write!(f, "{str}")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Bound {
    AtLeast(i32),
    AtMost(i32),
}

impl Display for Bound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Bound::AtLeast(n) => format!("at least {n}"),
            Bound::AtMost(n) => format!("at most {n}"),
        };

        write!(f, "{str}")
    }
}

impl Bound {
    // Relaxes (makes it weaker) a bound by amount.
    // Panics if the resulting bound exceeds the i32 range.
    fn relax(self, amount: u32) -> Bound {
        let amount = i64::from(amount);
        match self {
            AtLeast(n) => {
                let value = i64::from(n)
                    .checked_sub(amount)
                    .expect("Cannot weaken HalfPlane clue: bound overflow");

                AtLeast(
                    i32::try_from(value).expect("Cannot weaken HalfPlane clue: bound out of range"),
                )
            }
            AtMost(n) => {
                let value = i64::from(n)
                    .checked_add(amount)
                    .expect("Cannot weaken HalfPlane clue: bound overflow");

                AtMost(
                    i32::try_from(value).expect("Cannot weaken HalfPlane clue: bound out of range"),
                )
            }
        }
    }
}

impl Clue {
    /// TODO: missing docs for "holds"
    // True when this clue is consistent with the treasure lying in ‘cell‘.
    pub fn holds(self, cell: Cell) -> bool {
        match self {
            Clue::Within { centre, radius } => cell.distance(centre) <= radius,
            Clue::Beyond { centre, radius } => cell.distance(centre) > radius,
            Clue::HalfPlane { axis, bound } => {
                let v = match axis {
                    Axis::Q => cell.q,
                    Axis::R => cell.r,
                    Axis::S => cell.s(),
                };
                match bound {
                    Bound::AtLeast(k) => v >= k,
                    Bound::AtMost(k) => v <= k,
                }
            }
            Clue::Nearer { a, b } => cell.distance(a) < cell.distance(b),
        }
    }

    /// TODO: missing docs for "weaken"
    pub fn weaken(self, map: &impl Map, times: u32) -> Clue {
        match self {
            Clue::Within { centre, radius } => {
                let new_radius = radius
                    .checked_add(times)
                    .expect("Cannot weaken Within clue: radius overflow");

                Clue::Within {
                    centre,
                    radius: new_radius,
                }
            },
            Clue::Beyond { centre, radius } => {
                let new_radius = radius
                    .checked_sub(times)
                    .expect("Cannot weaken Beyond clue: radius underflow");

                Clue::Beyond {
                    centre,
                    radius: new_radius,
                }
            },
            Clue::HalfPlane { axis, bound } => Clue::HalfPlane {
                axis,
                bound: bound.relax(times),
            },
            Clue::Nearer { a, b } => {
                assert!(
                    times >= 2,
                    "Cannot weaken Nearer clue: times must be at least 2"
                );

                assert!(
                    a != b,
                    "Cannot weaken Nearer clue: cells A and B are identical"
                );

                // Calculate the direction from A to B.
                let direction_q = i64::from(b.q) - i64::from(a.q);
                let direction_r = i64::from(b.r) - i64::from(a.r);

                // Calculate the new position using the requested multiplier
                let k = i64::from(times);

                let new_q = i64::from(a.q)
                    .checked_add(
                        k.checked_mul(direction_q)
                            .expect("Cannot weaken Nearer clue: overflow"),
                    )
                    .expect("Cannot weaken Nearer clue: overflow");

                let new_r = i64::from(a.r)
                    .checked_add(
                        k.checked_mul(direction_r)
                            .expect("Cannot weaken Nearer clue: overflow"),
                    )
                    .expect("Cannot weaken Nearer clue: overflow");

                // Convert the coordinates back to i32
                let new_b = Cell {
                    q: i32::try_from(new_q)
                        .expect("Cannot weaken Nearer clue: q coordinate out of range"),
                    r: i32::try_from(new_r)
                        .expect("Cannot weaken Nearer clue: r coordinate out of range"),
                };

                // The new cell must belong to the map
                assert!(
                    map.contains(new_b),
                    "Cannot weaken Nearer clue: new cell is outside the map"
                );

                Clue::Nearer { a, b: new_b }
            }
        }
    }

    // returns true if self is weaker than other
    // if self is atleast as strong as other -> returns false
    // returns Err if the two clues are of different types
    // useful for ships that will be trying to figure out if the clue they received is a weaker than
    // a previous one
    pub fn is_weaker_than(self, other: Clue) -> Result<bool, String> {
        match (self, other) {
            //Two within clues
            (
                Clue::Within {
                    centre: c1,
                    radius: r1,
                },
                Clue::Within {
                    centre: c2,
                    radius: r2,
                },
            ) if c1 == c2 => Ok(r1 > r2),

            //Two beyond clues
            (
                Clue::Beyond {
                    centre: c1,
                    radius: r1,
                },
                Clue::Beyond {
                    centre: c2,
                    radius: r2,
                },
            ) if c1 == c2 => Ok(r1 < r2),

            //Two halfplane clues
            (
                Clue::HalfPlane {
                    axis: a1,
                    bound: b1,
                },
                Clue::HalfPlane {
                    axis: a2,
                    bound: b2,
                },
            ) if a1 == a2 => match (b1, b2) {
                (AtLeast(first), AtLeast(second)) => Ok(first < second),
                (AtMost(first), AtMost(second)) => Ok(first > second),
                _ => Err("The two Clues are not compatible".into()),
            },

            //Two nearer clues
            (Clue::Nearer { a: a1, b: b1 }, Clue::Nearer { a: a2, b: b2 }) if a1 == a2 => {
                Ok(a1.distance(b1) > a1.distance(b2))
            }

            _ => Err("can't comptare the two clues".into()),
        }
    }
}

impl Display for Clue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Clue::Within { centre, radius } => format!(
                "The treasure is within {} units of ({},{},{}) cell",
                radius,
                centre.q,
                centre.r,
                centre.s(),
            ),
            Clue::Beyond { centre, radius } => format!(
                "The treasure is beyond {} units from the ({},{},{}) cell",
                radius,
                centre.q,
                centre.r,
                centre.s(),
            ),
            Clue::HalfPlane { axis, bound } => {
                format!("The treasure is {} on the {} axis", bound, axis)
            }
            Clue::Nearer { a, b } => format!(
                "The treasure is nearer to ({}, {}, {}) than to ({}, {}, {})",
                a.q,
                a.r,
                a.s(),
                b.q,
                b.r,
                b.s()
            ),
        };

        write!(f, "{str}")
    }
}

// TODO: tests
#[cfg(test)]
pub mod tests;
