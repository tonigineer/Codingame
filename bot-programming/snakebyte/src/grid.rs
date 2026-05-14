use crate::position::Position;
use std::ops::{Index, IndexMut};

/// A rectangular grid of elements stored in row-major order.
///
/// The grid has a fixed [`width`](Grid::width) and [`height`](Grid::height),
/// and its elements are stored in a flat [`Vec<T>`] in row-major order:
/// all elements of row `0` first, followed by row `1`, etc.
#[derive(Clone)]
pub struct Grid<T> {
    width: i32,
    height: i32,
    bytes: Vec<T>,
}

impl Grid<u8> {
    /// Prints the contents of the grid to standard output.
    ///
    /// Each cell is interpreted as a `u8` character and printed in row-major order.
    /// A newline is inserted after each row, and an extra blank line is added after the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::{grid::*,position::*};
    ///
    /// let grid = Grid::from("ab\ncd");
    /// grid.print();
    /// // ab
    /// // cd
    /// //
    /// ```
    pub fn print(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let p = Position::new(x, y);
                eprint!("{}", self[p] as char);
            }
            println!();
        }
        println!();
    }
}
impl From<&str> for Grid<u8> {
    /// Creates a [`Grid<u8>`] from a multi-line string input.
    ///
    /// Each line must have the same length. Characters are stored as raw bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::{grid::*,position::*};
    ///
    /// let grid = Grid::from("abc\ndef");
     /// assert_eq!(grid.width(), 3);
     /// assert_eq!(grid.height(), 2);
     /// assert_eq!(grid[Position::new(0, 0)], b'a');
     /// assert_eq!(grid[Position::new(1, 1)], b'e');
     /// ```
       fn from(input: &str) -> Self {
           let raw: Vec<_> = input.lines().map(str::as_bytes).collect();
           let width = i32::try_from(raw[0].len()).expect("width too large");
           let height = i32::try_from(raw.len()).expect("height too large");
           let mut bytes = Vec::with_capacity((width as usize) * (height as usize));
          for slice in &raw {
              bytes.extend_from_slice(slice);
          }

        Grid {
            width,
            height,
            bytes,
        }
    }
}

impl From<String> for Grid<u8> {
    /// Creates a [`Grid<u8>`] from a multi-line string input.
    ///
    /// Each line must have the same length. Characters are stored as raw bytes.
    ///
    /// # Examples
    ///
    /// ```
     /// use aoc::common::{grid::*,position::*};
     ///
     /// let grid = Grid::from("abc\ndef".to_string());
     /// assert_eq!(grid.width(), 3);
     /// assert_eq!(grid.height(), 2);
     /// assert_eq!(grid[Position::new(0, 0)], b'a');
     /// assert_eq!(grid[Position::new(1, 1)], b'e');
     /// ```
       fn from(input: String) -> Self {
           let raw: Vec<_> = input.lines().map(str::as_bytes).collect();
           let width = i32::try_from(raw[0].len()).expect("width too large");
           let height = i32::try_from(raw.len()).expect("height too large");
           let mut bytes = Vec::with_capacity((width as usize) * (height as usize));
          for slice in &raw {
              bytes.extend_from_slice(slice);
          }

        Grid {
            width,
            height,
            bytes,
        }
    }
}

impl<T> Grid<T> {
    /// Returns the width of the grid (number of columns).
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::{grid::*,position::*};
    ///
    /// let grid = Grid::from("abc\ndef");
    /// assert_eq!(grid.width(), 3);
    /// ```
    #[inline]
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Returns the height of the grid (number of rows).
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::{grid::*,position::*};
    ///
    /// let grid = Grid::from("abc\ndef");
    /// assert_eq!(grid.height(), 2);
    /// ```
    #[inline]
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Returns `true` if the given [`Position`] lies inside the grid’s bounds.
    ///
    /// A position is inside the grid if its `x` coordinate is between
    /// `0` and `width - 1`, and its `y` coordinate is between
    /// `0` and `height - 1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::{grid::*,position::*};
    ///
    /// let grid = Grid::from("abc\ndef");
    /// assert!(grid.contains(Position::new(0, 0)));
    /// assert!(!grid.contains(Position::new(5, 5)));
    /// ```
    #[inline]
    pub fn contains(&self, position: Position) -> bool {
        position.x >= 0 && position.x < self.width && position.y >= 0 && position.y < self.height
    }
}

impl<T: PartialEq + Copy> Grid<T> {
    /// Searches the grid for the first occurrence of `token` and returns its [`Position`].
    ///
    /// The search is performed in row-major order, starting from the top-left cell `(0, 0)`.
    /// If the value is found, the coordinates of its position are returned; otherwise, `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::{grid::*,position::*};
    ///
    /// let grid = Grid::from("abc\ndef");
    /// assert_eq!(grid.search(b'e'), Some(Position::new(1, 1)));
    /// assert_eq!(grid.search(b'z'), None);
    /// ```
    #[inline]
    pub fn search(&self, token: T) -> Option<Position> {
        self.bytes
            .iter()
            .position(|h| *h == token)
            .map(|i| Position::new((i as i32) % self.width, (i as i32) / self.width))
    }
}

impl<T> Index<Position> for Grid<T> {
    type Output = T;

    /// Returns a shared reference to the element at the given [`Position`].
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::{grid::*,position::*};
    ///
    /// let grid = Grid::from("ab\ncd");
    /// assert_eq!(grid[Position::new(0, 0)], b'a');
    /// assert_eq!(grid[Position::new(1, 1)], b'd');
    /// ```
    #[inline]
    fn index(&self, index: Position) -> &Self::Output {
        &self.bytes[self.width as usize * index.y as usize + index.x as usize]
    }
}

impl<T> IndexMut<Position> for Grid<T> {
    /// Returns a mutable reference to the element at the given [`Position`].
    ///
    /// # Examples
    ///
    /// ```
    /// use aoc::common::{grid::*,position::*};
    ///
    /// let mut grid = Grid::from("ab\ncd");
    /// grid[Position::new(0, 0)] = b'x';
    /// assert_eq!(grid[Position::new(0, 0)], b'x');
    /// ```
    #[inline]
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self.bytes[(self.width * index.y + index.x) as usize]
    }
}
