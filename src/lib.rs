use std::fmt::{Display, format};

use crate::Bound::{AtLeast, AtMost};

// TODO: logs
// TODO: run pedantic clippy

// TODO: use Geometry's Cell
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[deprecated(note = "Use Geometry's Cell implementation")]
pub struct Cell {
    pub q: i32,
    pub r: i32,
}

impl Cell {
    pub fn s(&self) -> i32 {
        -self.q - self.r
    }

    pub fn distance(self, other: Cell) -> u32 {
        // calculating Delta q / r / s
        let dq = self.q.abs_diff(other.q);
        let dr = self.r.abs_diff(other.r);
        let ds = self.s().abs_diff(other.s());

        dq.checked_add(dr)
            .and_then(|sum| sum.checked_add(ds))
            .expect("distance overflow u32")
            / 2
    }
}

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
    // checks whethever a given bound contains val
    fn holds_val(self, val: i32) -> bool {
        todo!();
    }

    // relaxes (makes it weaker) a bound by amount
    fn relax(self, amount: u32) -> Bound {
        match self {
            AtLeast(n) => Bound::AtLeast(n - amount as i32),
            AtMost(n) => Bound::AtMost(n + amount as i32),
        }
    }
}

impl Clue {
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

    // returns a weaker version of a clue
    // 2 properties:
    // - every cell satisfying the current Clue must satisfy the weaker version as well
    // - weaken always produces a weaker clue of the same type
    // useful for islands which may produce clues that are n times weaker then the full one
    // also allows the island to gradually strengthen the clue
    // a helper for island
    // a helper for islands, prevents them from issuing clues of differe
    pub fn weaken(self, times: u32) -> Clue {
        match self {
            Clue::Within { centre, radius } => Clue::Within {
                centre,
                radius: radius + times,
            },
            Clue::Beyond { centre, radius } => Clue::Beyond {
                centre,
                radius: radius - times,
            },
            Clue::HalfPlane { axis, bound } => Clue::HalfPlane {
                axis,
                bound: bound.relax(times),
            },
            // TODO: finish nearer
            Clue::Nearer { a, b } => Clue::Nearer { a, b },
        }
    }

    // returns true if self is weaker than other
    // if self is atleast as strong as other -> returns false
    // returns Err if the two clues are of different types
    // useful for ships that will be trying to figure out if the clue they received is a weaker than
    // a previous one
    pub fn is_weaker_than(self, other: Clue) -> Result<bool, String> {
        match self {
            Clue::Within { centre, radius } => {
                if let Clue::Within {
                    centre: c,
                    radius: r,
                } = other
                {
                    if centre != c {
                        return Err("Cannot compare Within with different centers".into());
                    }
                    return Ok(radius < r);
                }
            }

            Clue::Beyond { centre, radius } => {
                if let Clue::Beyond {
                    centre: c,
                    radius: r,
                } = other
                {
                    if centre != c {
                        return Err("Cannot compare Beyond with different centers".into());
                    }
                    return Ok(radius >= r);
                }
            }

            // TODO: finish HalfPlane
            Clue::HalfPlane { axis, bound } => {
                if let Clue::HalfPlane { axis: a, bound: b } = other {
                    if axis != a || bound != b {
                        return Err("Cannot compare HalfPlane with different axis / bound".into());
                    }

                    return Ok(true);
                }
            }

            // TODO: finish Nearer
            Clue::Nearer { a: a2, b: b2 } => {}
        };
        Err("The clues are of different types".into())
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
pub mod tests {
    use super::*; // brings everything from clues to tests

    #[test]
    fn display_variants() {
        let c1 = Cell { q: 1, r: -2 };
        let c2 = Cell { q: 17, r: 91 };
        let cases = [
            (
                Clue::Within {
                    centre: c1,
                    radius: 3,
                },
                "The treasure is within 3 units of (1,-2,1) cell",
            ),
            (
                Clue::Beyond {
                    centre: c1,
                    radius: 3,
                },
                "The treasure is beyond 3 units from the (1,-2,1) cell",
            ),
            (
                Clue::HalfPlane {
                    axis: Axis::Q,
                    bound: Bound::AtLeast(4),
                },
                "The treasure is at least 4 on the Q axis",
            ),
            (
                Clue::HalfPlane {
                    axis: Axis::S,
                    bound: Bound::AtMost(-2),
                },
                "The treasure is at most -2 on the S axis",
            ),
            (
                Clue::Nearer { a: c1, b: c2 },
                "The treasure is nearer to (1, -2, 1) than to (17, 91, -108)",
            ),
        ];

        for (clue, expected) in cases {
            assert_eq!(clue.to_string(), expected);
        }
    }
}
