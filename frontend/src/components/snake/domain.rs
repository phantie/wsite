#![allow(unused)]

pub struct Snake {
    pub sections: Sections,
    // direction snake will move on advance, always valid
    pub direction: Direction,
}

pub enum AdvanceResult {
    Success,
    BitYaSelf,
}

impl Snake {
    fn rm_tail(&mut self) {
        self.sections.rm_tail();
    }

    // head section, see mouth
    fn head(&self) -> Section {
        self.sections.head()
    }

    pub fn mouth(&self) -> Pos {
        self.head().end()
    }

    pub fn iter_vertices(&self) -> impl Iterator<Item = Pos> + '_ {
        self.sections
            .as_ref()
            .iter()
            .map(|section| section.start())
            .chain(std::iter::once(self.head().end()))
    }

    fn bit_ya_self(&self, advanced_head: Section) -> bool {
        // all sections except tail, because it won't be here when head advances
        self.iter_vertices()
            .skip(1)
            .find(|pos| pos == &advanced_head.end())
            .is_some()
    }

    fn advance_head(&mut self) -> AdvanceResult {
        let advanced_head = self.head().next(self.direction).unwrap();

        if self.bit_ya_self(advanced_head) {
            AdvanceResult::BitYaSelf
        } else {
            self.sections.push_head(self.direction).unwrap();
            AdvanceResult::Success
        }
    }

    pub fn advance(&mut self, foods: &mut Foods) -> AdvanceResult {
        match self.advance_head() {
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
            AdvanceResult::BitYaSelf => AdvanceResult::BitYaSelf,
        }
    }

    pub fn set_direction(&mut self, direction: Direction) -> Result<(), ()> {
        // forbid direction opposite to the direction of the head
        if self.head().is_opposite_direction(direction) {
            Err(())
        } else {
            self.direction = direction;
            Ok(())
        }
    }

    pub fn boundaries(&self) -> Boundaries {
        let snake = self
            .sections
            .as_ref()
            .iter()
            .map(|section| [section.start(), section.end()])
            .flatten();
        Boundaries::from_iterators(snake.clone().map(Pos::x), snake.map(Pos::y)).unwrap()
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

    pub fn pos(&self) -> Pos {
        self.pos
    }
}

impl From<Pos> for Food {
    fn from(pos: Pos) -> Self {
        Self { pos }
    }
}

#[derive(Debug)]
pub struct Foods {
    pub values: Vec<Food>,
}

impl AsRef<Vec<Food>> for Foods {
    fn as_ref(&self) -> &Vec<Food> {
        &self.values
    }
}

impl Foods {
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
            .find(|(_i, f)| f.pos == pos)
            .expect("to call only when such element exists");
        self.values.remove(i);
    }

    pub fn boundaries(&self) -> Option<Boundaries> {
        let foods = self.as_ref().iter().map(Food::pos);
        Boundaries::from_iterators(foods.clone().map(Pos::x), foods.map(Pos::y))
    }
}

#[derive(Clone, Copy)]
pub struct Vector {
    pub start: Pos,
    pub end: Pos,
}

impl Vector {
    // determine section direction
    // line formed must be parallel to the horizon or vertical
    pub fn direction(&self) -> Result<Direction, ()> {
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

pub struct Sections {
    sections: Vec<Section>,
}

impl Sections {
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    pub fn head(&self) -> Section {
        self.as_ref().last().unwrap().clone()
    }

    fn rm_tail(&mut self) {
        self.as_mut().remove(0);
    }

    fn push_head(&mut self, direction: Direction) -> Result<(), ()> {
        let advanced_head = self.head().next(direction);

        match advanced_head {
            Ok(advanced_head) => {
                self.as_mut().push(advanced_head);
                Ok(())
            }
            Err(()) => Err(()),
        }
    }

    pub fn from_directions(
        initial_pos: Pos,
        directions: impl IntoIterator<Item = Direction>,
    ) -> Self {
        let mut directions = directions.into_iter();

        let initial_section = Section::initial(
            initial_pos,
            directions.next().expect("to form at least one section"),
        );
        let mut sections = vec![initial_section];

        for direction in directions {
            sections.push(
                sections
                    .last()
                    .unwrap()
                    .next(direction)
                    .expect("two subsequent directions not to opposite"),
            );
        }

        Self { sections }
    }
}

impl AsRef<Vec<Section>> for Sections {
    fn as_ref(&self) -> &Vec<Section> {
        &self.sections
    }
}

impl AsMut<Vec<Section>> for Sections {
    fn as_mut(&mut self) -> &mut Vec<Section> {
        &mut self.sections
    }
}

#[derive(Clone, Copy)]
pub struct Section {
    vector: Vector,
}

impl Section {
    fn new(start: Pos, end: Pos) -> Self {
        Self {
            vector: Vector { start, end },
        }
    }

    pub fn start(&self) -> Pos {
        self.vector.start
    }

    pub fn end(&self) -> Pos {
        self.vector.end
    }

    pub fn initial(start: Pos, direction: Direction) -> Self {
        Self::new(start, start.to(direction))
    }

    pub fn direction(&self) -> Direction {
        self.vector.direction().unwrap()
    }

    pub fn is_opposite_direction(&self, direction: Direction) -> bool {
        self.direction().opposite() == direction
    }

    pub fn next(&self, direction: Direction) -> Result<Self, ()> {
        if self.is_opposite_direction(direction) {
            Err(())
        } else {
            Ok(Self::new(self.end(), self.end().to(direction)))
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl std::ops::Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to(&self, direction: Direction) -> Self {
        let (x_diff, y_diff) = match direction {
            Direction::Right => (1, 0),
            Direction::Left => (-1, 0),
            Direction::Bottom => (0, 1),
            Direction::Up => (0, -1),
        };

        Self {
            x: self.x + x_diff,
            y: self.y + y_diff,
        }
    }

    pub fn x(self) -> i32 {
        self.x
    }

    pub fn y(self) -> i32 {
        self.y
    }

    pub fn boundaries_in_radius(self, radius: i32) -> Boundaries {
        Boundaries {
            min: Pos::new(self.x - radius, self.y - radius),
            max: Pos::new(self.x + radius, self.y + radius),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Direction {
    Up,
    Bottom,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Bottom,
            Self::Bottom => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Boundaries {
    pub min: Pos,
    pub max: Pos,
}

impl Boundaries {
    fn from_iterators(
        xs: impl Iterator<Item = i32> + Clone,
        ys: impl Iterator<Item = i32> + Clone,
    ) -> Option<Boundaries> {
        let max_x = xs.clone().max()?;
        let min_x = xs.min().unwrap();
        let max_y = ys.clone().max()?;
        let min_y = ys.min().unwrap();

        Some(Boundaries {
            min: Pos { x: min_x, y: min_y },
            max: Pos { x: max_x, y: max_y },
        })
    }

    pub fn join(self, other: Self) -> Self {
        Self {
            min: Pos {
                x: self.min.x.min(other.min.x),
                y: self.min.y.min(other.min.y),
            },
            max: Pos {
                x: self.max.x.max(other.max.x),
                y: self.max.y.max(other.max.y),
            },
        }
    }

    pub fn join_option(self, other: Option<Self>) -> Self {
        match other {
            Some(other) => self.join(other),
            None => self,
        }
    }

    pub fn right_top(&self) -> Pos {
        Pos::new(self.max.x, self.min.y)
    }

    pub fn left_top(&self) -> Pos {
        self.min
    }

    pub fn left_bottom(&self) -> Pos {
        Pos::new(self.min.x, self.max.y)
    }

    pub fn right_bottom(&self) -> Pos {
        self.max
    }

    pub fn relation(&self, pos: Pos) -> RelationToBoundaries {
        use std::cmp::Ordering::*;
        use RelationToBoundaries::*;

        match [
            pos.x.cmp(&self.min.x),
            pos.y.cmp(&self.min.y),
            pos.x.cmp(&self.max.x),
            pos.y.cmp(&self.max.y),
        ] {
            [Greater, Greater, Less, Less] => Inside,
            arr if arr.iter().any(|r| r == &Equal) => Touching,
            _ => Outside,
        }
    }
}

pub enum RelationToBoundaries {
    Inside,
    Touching,
    Outside,
}
