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
    BitYaSelf,
}

impl Snake {
    fn rm_tail(&mut self) {
        self.sections.remove(0);
    }

    // head section, see mouth
    fn head(&self) -> &Section {
        self.sections.last().unwrap()
    }

    pub fn mouth(&self) -> Pos {
        self.head().end
    }

    pub fn iter_vertices(&self) -> impl Iterator<Item = Pos> + '_ {
        self.sections
            .iter()
            .map(|s| s.start.clone())
            .chain(std::iter::once(self.sections.last().unwrap().end.clone()))
    }

    fn bit_ya_self(&self, advanced_head: Section) -> bool {
        self.iter_vertices()
            .skip(1)
            .find(|p| p == &advanced_head.end)
            .is_some()
    }

    fn advance_head(&mut self, w: WindowSize) -> AdvanceResult {
        let advanced_head = self.head().next(self.direction);

        if advanced_head.end.out_of_window_bounds(w) {
            AdvanceResult::OutOfBounds
        } else if
        // all sections except tail, because it won't be here when head advances
        self.bit_ya_self(advanced_head) {
            AdvanceResult::BitYaSelf
        } else {
            self.sections.push(advanced_head);
            AdvanceResult::Success
        }
    }

    pub fn advance(&mut self, w: WindowSize, foods: &mut Foods) -> AdvanceResult {
        match self.advance_head(w) {
            AdvanceResult::Success => {
                // if on next step mouth will eat food -
                // remove food and don't remove tail
                if foods.has_pos(self.mouth()) {
                    foods.remove_with_pos(self.mouth());
                } else {
                    self.rm_tail();
                }
                AdvanceResult::Success
            }
            AdvanceResult::OutOfBounds => AdvanceResult::OutOfBounds,
            AdvanceResult::BitYaSelf => AdvanceResult::BitYaSelf,
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

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Food {
    pub pos: Pos,
}

impl Food {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            pos: Pos::new(x, y),
        }
    }
}

#[derive(Debug)]
pub struct Foods {
    values: Vec<Food>,
}

impl Default for Foods {
    fn default() -> Self {
        Self::init()
    }
}

impl AsRef<Vec<Food>> for Foods {
    fn as_ref(&self) -> &Vec<Food> {
        &self.values
    }
}

impl Foods {
    pub fn init() -> Self {
        let values = vec![
            Food::new(200, 500),
            Food::new(300, 600),
            Food::new(600, 300),
            Food::new(700, 400),
        ];

        Self { values }
    }

    pub fn has_pos(&self, pos: Pos) -> bool {
        self.values
            .iter()
            .map(|v| v.pos)
            .collect::<Vec<_>>()
            .contains(&pos)
    }

    pub fn remove_with_pos(&mut self, pos: Pos) {
        let (i, _food) = self
            .values
            .iter()
            .enumerate()
            .find(|(i, f)| f.pos == pos)
            .expect("to call only when such element exists");
        self.values.remove(i);
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
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
