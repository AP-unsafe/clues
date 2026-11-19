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
