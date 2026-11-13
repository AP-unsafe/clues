use std::fmt::Display;

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
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Bound {
    AtLeast(i32),
    AtMost(i32),
}

impl Bound {
    // checks whethever a given bound contains val
    pub fn holds_val(self, val: i32) -> bool {
        todo!();
    }

    // relaxes a bound by amount
    pub fn relax(self, amount: i32) -> Bound {
        todo!()
    }
}

impl Clue {
    // True when this clue is consistent with the treasure lying in ‘cell‘.
    // TODO: deal with overflows here
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
    pub fn weaken(self, times: u8) -> Clue {
        todo!()
    }

    // returns true if self is weaker than other, otherwise returns false
    // returns Err if the two clues are of different types
    // useful for ships that will be trying to figure out if the clue they received is a weaker than
    // a previous one
    pub fn is_weaker_than(self, other: Clue) -> Result<bool, String> {
        todo!()
    }
}

impl Display for Clue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

// TODO: tests
#[cfg(test)]
pub mod tests {
    use super::*; // brings everything from clues to tests

    #[test]
    fn it_works() {
        todo!();
    }
}
