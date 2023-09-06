#![allow(unused)]

const SECTION_LENGTH: u32 = 100;
const STARTING_PARTS: u32 = 3;

pub struct Snake {
    pub sections: Vec<Section>,
}

impl Default for Snake {
    fn default() -> Self {
        let initial_pos = Pos::new(100, 100);

        let initial_section = Section::initial(initial_pos, initial_pos.to_bottom());
        let second_section = initial_section.next(Direction::Right);
        let third_section = second_section.next(Direction::Bottom);

        let sections = vec![initial_section, second_section, third_section];
        Self { sections }
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

pub enum Direction {
    Up,
    Bottom,
    Left,
    Right,
}
