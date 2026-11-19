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
    // relaxes (makes it weaker) a bound by amount
    fn relax(self, amount: u32) -> Option<Bound> {
        let amount = i64::from(amount);
        match self {
            AtLeast(n) => {
                let value = i64::from(n).checked_sub(amount)?;
                Some(AtLeast(i32::try_from(value).ok()?))
            }
            AtMost(n) => {
                let value = i64::from(n).checked_add(amount)?;
                Some(AtMost(i32::try_from(value).ok()?))
            }
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

    pub fn weaken(self, map: &Map, times: u32) -> Option<Clue> {
        match self {
            Clue::Within { centre, radius } => {
                let new_radius = radius.checked_add(times)?;

                Some(Clue::Within {
                    centre,
                    radius: new_radius,
                })
            }
            Clue::Beyond { centre, radius } => {
                let new_radius = radius.checked_sub(times)?;

                Some(Clue::Beyond {
                    centre,
                    radius: new_radius,
                })
            }
            Clue::HalfPlane { axis, bound } => Some(Clue::HalfPlane {
                axis,
                bound: bound.relax(times)?,
            }),
            Clue::Nearer { a, b } => {
                // if a == b -> can't calculate the direction
                if times < 2 || a == b {
                    return None;
                }

                // Calculate the direction from A to B
                let direction_q = i64::from(b.q) - i64::from(a.q);
                let direction_r = i64::from(b.r) - i64::from(a.r);

                // Generate all valid candidate cells inside the map
                let mut candidates: Vec<(u32, Cell)> = Vec::new();

                // k indicates how far along the axis between A and B the candidate cell is
                let mut k: u32 = 2;

                loop {
                    // Calculate candidate coordinates using checked arithmetic
                    let k_i64 = i64::from(k);

                    let q = i64::from(a.q).checked_add(k_i64.checked_mul(direction_q)?)?;

                    let r = i64::from(a.r).checked_add(k_i64.checked_mul(direction_r)?)?;

                    // Convert coordinates back to i32
                    let candidate = Cell {
                        q: i32::try_from(q).ok()?,
                        r: i32::try_from(r).ok()?,
                    };

                    // Stop when the candidate leaves the map
                    if !map.contains(candidate) {
                        break;
                    }

                    candidates.push((k, candidate));

                    // Avoid overflowing the multiplier
                    k = k.checked_add(1)?;
                }

                // Select the candidate corresponding to the requested multiplier
                let new_b = candidates
                    .iter()
                    .find(|(candidate_k, _)| *candidate_k == times)
                    .map(|(_, cell)| *cell)?;

                // Return the weakened clue
                Some(Clue::Nearer { a, b: new_b })
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
pub mod tests {
    use super::*; // brings everything from clues to tests

    // a small helper to create cells inline
    fn cell(q: i32, r: i32) -> Cell {
        Cell { q, r }
    }

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

    mod is_weaker_than_tests {
        use super::*;

        #[test]
        fn within_larger_radius_is_weaker() {
            let c = cell(0, 0);
            let weak = Clue::Within {
                centre: c,
                radius: 5,
            };
            let strong = Clue::Within {
                centre: c,
                radius: 2,
            };

            assert_eq!(weak.is_weaker_than(strong), Ok(true));
            assert_eq!(strong.is_weaker_than(weak), Ok(false));
        }

        #[test]
        fn beyond_smaller_radius_is_weaker() {
            let c = cell(0, 0);
            let weak = Clue::Beyond {
                centre: c,
                radius: 1,
            };
            let strong = Clue::Beyond {
                centre: c,
                radius: 4,
            };

            assert_eq!(weak.is_weaker_than(strong), Ok(true));
            assert_eq!(strong.is_weaker_than(weak), Ok(false));
        }

        #[test]
        fn half_plane_lower_at_least_bound_is_weaker() {
            let weak = Clue::HalfPlane {
                axis: Axis::Q,
                bound: Bound::AtLeast(1),
            };
            let strong = Clue::HalfPlane {
                axis: Axis::Q,
                bound: Bound::AtLeast(5),
            };

            assert_eq!(weak.is_weaker_than(strong), Ok(true));
            assert_eq!(strong.is_weaker_than(weak), Ok(false));
        }

        #[test]
        fn nearer_farther_b_is_weaker() {
            let a = cell(0, 0);
            let weak = Clue::Nearer { a, b: cell(3, 0) };
            let strong = Clue::Nearer { a, b: cell(1, 0) };

            assert_eq!(weak.is_weaker_than(strong), Ok(true));
            assert_eq!(strong.is_weaker_than(weak), Ok(false));
        }

        #[test]
        fn equal_nearer_clues_are_not_weaker() {
            let clue = Clue::Nearer {
                a: cell(0, 0),
                b: cell(2, 0),
            };

            assert_eq!(clue.is_weaker_than(clue), Ok(false));
        }

        // ---- Same type, incompatible ----

        #[test]
        fn within_with_different_centres_fails() {
            let x = Clue::Within {
                centre: cell(0, 0),
                radius: 3,
            };
            let y = Clue::Within {
                centre: cell(1, 0),
                radius: 3,
            };

            assert!(x.is_weaker_than(y).is_err());
        }

        #[test]
        fn beyond_with_different_centres_fails() {
            let x = Clue::Beyond {
                centre: cell(0, 0),
                radius: 3,
            };
            let y = Clue::Beyond {
                centre: cell(1, 0),
                radius: 3,
            };

            assert!(x.is_weaker_than(y).is_err());
        }

        #[test]
        fn half_plane_with_different_axes_fails() {
            let x = Clue::HalfPlane {
                axis: Axis::Q,
                bound: Bound::AtLeast(1),
            };
            let y = Clue::HalfPlane {
                axis: Axis::R,
                bound: Bound::AtLeast(1),
            };

            assert!(x.is_weaker_than(y).is_err());
        }

        #[test]
        fn half_plane_with_opposite_bound_kinds_fails() {
            let x = Clue::HalfPlane {
                axis: Axis::Q,
                bound: Bound::AtLeast(1),
            };
            let y = Clue::HalfPlane {
                axis: Axis::Q,
                bound: Bound::AtMost(1),
            };

            assert!(x.is_weaker_than(y).is_err());
        }

        #[test]
        fn nearer_with_different_a_fails() {
            let x = Clue::Nearer {
                a: cell(0, 0),
                b: cell(2, 0),
            };
            let y = Clue::Nearer {
                a: cell(1, 0),
                b: cell(2, 0),
            };

            assert!(x.is_weaker_than(y).is_err());
        }

        // ---- Different types ----

        #[test]
        fn different_type_clues_cannot_be_compared() {
            let within = Clue::Within {
                centre: cell(0, 0),
                radius: 3,
            };
            let beyond = Clue::Beyond {
                centre: cell(0, 0),
                radius: 3,
            };
            let half = Clue::HalfPlane {
                axis: Axis::Q,
                bound: Bound::AtLeast(0),
            };
            let nearer = Clue::Nearer {
                a: cell(0, 0),
                b: cell(1, 0),
            };

            assert!(within.is_weaker_than(beyond).is_err());
            assert!(beyond.is_weaker_than(half).is_err());
            assert!(half.is_weaker_than(nearer).is_err());
            assert!(nearer.is_weaker_than(within).is_err());
        }
    }

    mod weaken_tests {
        use super::*;
    }
}
