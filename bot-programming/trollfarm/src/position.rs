use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub const ORIGIN: Position = Position { x: 0, y: 0 };
pub const RIGHT: Position = Position { x: 1, y: 0 };
pub const DOWN: Position = Position { x: 0, y: 1 };
pub const LEFT: Position = Position { x: -1, y: 0 };
pub const UP: Position = Position { x: 0, y: -1 };

pub const CARDINALS: [Position; 4] = [RIGHT, DOWN, LEFT, UP];
pub const DIAGONALS: [Position; 4] = [
    Position { x: 1, y: 1 },
    Position { x: -1, y: 1 },
    Position { x: -1, y: -1 },
    Position { x: 1, y: -1 },
];

/// A 2D position with integer coordinates.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Returns a new [`Position`] with the given `x` and `y` coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::position::*;
    ///
    /// let p = Position::new(3, 4);
    /// assert_eq!(p.x, 3);
    /// assert_eq!(p.y, 4);
    /// ```
    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the [Manhattan distance](https://en.wikipedia.org/wiki/Taxicab_geometry)
    /// between `self` and another [`Position`].
    ///
    /// The Manhattan distance is the sum of the absolute differences of the `x`
    /// and `y` coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::position::*;
    ///
    /// let a = Position::new(1, 2);
    /// let b = Position::new(4, 6);
    /// assert_eq!(a.manhattan(&b), 7);
    /// ```
    #[inline]
    pub fn manhattan(&self, other: &Self) -> usize {
        self.x.abs_diff(other.x) as usize + self.y.abs_diff(other.y) as usize
    }
}

impl Hash for Position {
    /// Feeds the `x` and `y` coordinates into the given [`Hasher`].
    ///
    /// This allows [`Position`] to be used as a key in hashed collections such as
    /// [`HashMap`](std::collections::HashMap) and [`HashSet`](std::collections::HashSet).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use aoc::common::position::Position;
    ///
    /// let mut set = HashSet::new();
    /// set.insert(Position::new(1, 2));
    /// assert!(set.contains(&Position::new(1, 2)));
    /// ```
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.x as u32);
        state.write_u32(self.y as u32);
    }
}

impl Add for Position {
    type Output = Self;

    /// Adds two positions component-wise.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::position::*;
    ///
    /// let a = Position::new(1, 2);
    /// let b = Position::new(3, 4);
    /// assert_eq!(a + b, Position::new(4, 6));
    /// ```
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Position::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Position {
    /// Adds another position to this one component-wise, in place.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::position::*;
    ///
    /// let mut p = Position::new(1, 2);
    /// p += Position::new(3, 4);
    /// assert_eq!(p, Position::new(4, 6));
    /// ```
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Position {
    type Output = Self;

    /// Subtracts one position from another component-wise.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::position::*;
    ///
    /// let a = Position::new(5, 7);
    /// let b = Position::new(2, 3);
    /// assert_eq!(a - b, Position::new(3, 4));
    /// ```
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Position::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Position {
    /// Subtracts another position from this one component-wise, in place.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::position::*;
    ///
    /// let mut p = Position::new(5, 7);
    /// p -= Position::new(2, 3);
    /// assert_eq!(p, Position::new(3, 4));
    /// ```
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<i32> for Position {
    type Output = Self;

    /// Multiplies both coordinates of the position by a scalar.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::position::*;
    ///
    /// let p = Position::new(2, 3);
    /// assert_eq!(p * 4, Position::new(8, 12));
    /// ```
    #[inline]
    fn mul(self, rhs: i32) -> Self {
        Position::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<i32> for Position {
    type Output = Self;

    /// Divides both coordinates of the position by a scalar (integer division).
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::position::*;
    ///
    /// let p = Position::new(8, 12);
    /// assert_eq!(p / 4, Position::new(2, 3));
    /// ```
    #[inline]
    fn div(self, rhs: i32) -> Self {
        assert!(rhs != 0, "division by zero");
        Position::new(self.x / rhs, self.y / rhs)
    }
}
