#![allow(unused)]

const SECTION_LENGTH: u32 = 100;
const STARTING_PARTS: u32 = 3;

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

        // TODO auto derive initial direction for head direction
        let direction = Direction::Right;
        Self {
            sections,
            direction,
        }
    }
}

impl Snake {
    fn rm_tail(&mut self) {
        self.sections.remove(0);
    }

    fn head(&self) -> &Section {
        self.sections.last().unwrap()
    }

    fn advance_head(&mut self) {
        let s = self.head().next(self.direction);
        self.sections.push(s);
    }

    pub fn advance(&mut self) {
        self.rm_tail();
        self.advance_head();
    }

    pub fn set_direction(&mut self, direction: Direction) -> Result<(), ()> {
        if self.direction.opposite() == direction {
            Err(())
        } else {
            self.direction = direction;
            Ok(())
        }
    }
}

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
}

#[derive(Copy, Clone)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

impl Pos {
    fn new(x: u32, y: u32) -> Self {
        Self { x, y }
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
