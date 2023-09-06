#![allow(unused)]

use super::common::WindowSize;

const SECTION_LENGTH: i32 = 100;
const STARTING_PARTS: i32 = 3;

pub struct Snake {
    pub sections: Vec<Section>,
    // direction snake will move on advance
    pub direction: Direction,
}

impl Default for Snake {
    fn default() -> Self {
        let initial_pos = Pos::new(100, 100);

        let initial_section = Section::initial(initial_pos, initial_pos.to_bottom());
        let second_section = initial_section.next(Direction::Right);
        let head_section = second_section.next(Direction::Bottom);

        let sections = vec![initial_section, second_section, head_section];
        assert!(sections.len() >= 2, "snake must have at least ... sections");

        sections
            .iter()
            .map(|s| {
                assert!(
                    s.direction().is_ok(),
                    "invalid section, cannot determine direction"
                )
            })
            .collect::<Vec<_>>();

        // continue moving in the same direction
        let direction = head_section.direction().unwrap();

        Self {
            sections,
            direction,
        }
    }
}

pub enum AdvanceResult {
    Success,
    OutOfBounds,
}

impl Snake {
    fn rm_tail(&mut self) {
        self.sections.remove(0);
    }

    fn head(&self) -> &Section {
        self.sections.last().unwrap()
    }

    fn advance_head(&mut self, w: WindowSize) -> AdvanceResult {
        let s = self.head().next(self.direction);

        if s.start.out_of_window_bounds(w) || s.end.out_of_window_bounds(w) {
            AdvanceResult::OutOfBounds
        } else {
            self.sections.push(s);
            AdvanceResult::Success
        }
    }

    pub fn advance(&mut self, w: WindowSize) -> AdvanceResult {
        match self.advance_head(w) {
            AdvanceResult::Success => {
                self.rm_tail();
                AdvanceResult::Success
            }
            AdvanceResult::OutOfBounds => AdvanceResult::OutOfBounds,
        }
    }

    pub fn set_direction(&mut self, direction: Direction) -> Result<(), ()> {
        // forbid direction opposite to the direction of the head
        if self.head().direction().unwrap().opposite() == direction {
            Err(())
        } else {
            self.direction = direction;
            Ok(())
        }
    }
}

#[derive(Clone, Copy)]
pub struct Section {
    pub start: Pos,
    pub end: Pos,
}

impl Section {
    fn initial(start: Pos, end: Pos) -> Self {
        Self { start, end }
    }

    fn next(&self, direction: Direction) -> Self {
        Self {
            start: self.end,
            end: self.end.to(direction),
        }
    }

    // determine section direction
    // line formed must be parallel to the horizon or vertical
    fn direction(&self) -> Result<Direction, ()> {
        use std::cmp::Ordering;

        let horizontal = self.start.x.cmp(&self.end.x);
        let vertical = self.start.y.cmp(&self.end.y);

        match (horizontal, vertical) {
            (Ordering::Equal, Ordering::Greater) => Ok(Direction::Up),
            (Ordering::Equal, Ordering::Less) => Ok(Direction::Bottom),
            (Ordering::Greater, Ordering::Equal) => Ok(Direction::Left),
            (Ordering::Less, Ordering::Equal) => Ok(Direction::Right),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn out_of_window_bounds(&self, w: WindowSize) -> bool {
        self.x < 0 || self.y < 0 || self.x > w.width || self.y > w.height
    }

    fn to(&self, direction: Direction) -> Self {
        match direction {
            Direction::Right => self.to_right(),
            Direction::Left => self.to_left(),
            Direction::Bottom => self.to_bottom(),
            Direction::Up => self.to_up(),
        }
    }

    fn to_right(&self) -> Self {
        Self {
            x: self.x + SECTION_LENGTH,
            y: self.y,
        }
    }

    fn to_left(&self) -> Self {
        Self {
            x: self.x - SECTION_LENGTH,
            y: self.y,
        }
    }

    fn to_bottom(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + SECTION_LENGTH,
        }
    }

    fn to_up(&self) -> Self {
        Self {
            x: self.x,
            y: self.y - SECTION_LENGTH,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Bottom,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Bottom,
            Self::Bottom => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
