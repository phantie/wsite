#![allow(unused)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Domain {
    // your snake
    pub snake: Option<Snake>,
    pub other_snakes: Vec<Snake>,
    pub foods: Foods,
    pub boundaries: Boundaries,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Snake {
    pub sections: Sections,
    // direction snake will move on advance, always valid
    pub direction: Direction,
}

pub enum AdvanceResult {
    Success,
    BitSomeone,
    BitYaSelf,
    OutOfBounds,
}

impl Snake {
    fn rm_tail(&mut self) {
        self.sections.rm_tail();
    }

    // head section, see mouth
    fn head(&self) -> Section {
        self.sections.head()
    }

    fn tail(&self) -> Section {
        self.sections.tail()
    }

    pub fn mouth(&self) -> Pos {
        self.head().end()
    }

    pub fn tail_end(&self) -> Pos {
        self.sections.tail().start()
    }

    pub fn iter_vertices(&self) -> impl Iterator<Item = Pos> + '_ {
        self.sections.iter_vertices()
    }

    pub fn iter_vertices_without_tail(&self) -> impl Iterator<Item = Pos> + '_ {
        self.iter_vertices().skip(1)
    }

    fn bit_snake(&self, advanced_head: Section, myself: bool) -> bool {
        if myself {
            // all sections except tail, because it won't be here when head advances
            self.iter_vertices_without_tail()
                .find(|pos| pos == &advanced_head.end())
                .is_some()
        } else {
            let mut snake = self.clone();
            match snake.advance_head(
                &[],
                // TODO acts as a sentinel value
                &Boundaries {
                    min: Pos::new(-1000, -1000),
                    max: Pos::new(1000, 1000),
                },
            ) {
                AdvanceResult::Success => {}
                _ => unreachable!(),
            }
            let mut vertices = snake.iter_vertices_without_tail();
            vertices.find(|pos| pos == &advanced_head.end()).is_some()
        }
    }

    fn advance_head(&mut self, other_snakes: &[Snake], boundaries: &Boundaries) -> AdvanceResult {
        let advanced_head = self.head().next(self.direction).unwrap();

        // TODO duplicate logic
        let out_of_bounds = match boundaries.relation(advanced_head.end()) {
            RelationToBoundaries::Inside => false,
            RelationToBoundaries::Touching => true,
            RelationToBoundaries::Outside => true,
        };

        if out_of_bounds {
            AdvanceResult::OutOfBounds
        } else if self.bit_snake(advanced_head, true) {
            AdvanceResult::BitYaSelf
        } else if other_snakes
            .iter()
            .any(|snake| snake.bit_snake(advanced_head, false))
        {
            AdvanceResult::BitSomeone
        } else {
            self.sections.push_head(self.direction).unwrap();
            AdvanceResult::Success
        }
    }

    pub fn advance(
        &mut self,
        foods: &mut Foods,
        other_snakes: &[Snake],
        boundaries: &Boundaries,
    ) -> AdvanceResult {
        match self.advance_head(other_snakes, boundaries) {
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
            r => r,
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

    pub fn out_of_bounds(&self, boundaries: &Boundaries) -> bool {
        let mouth = self.mouth();
        match boundaries.relation(mouth) {
            RelationToBoundaries::Inside => false,
            RelationToBoundaries::Touching => true,
            RelationToBoundaries::Outside => true,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Foods {
    pub values: HashMap<Pos, Food>,
}

impl Serialize for Foods {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.values
            .values()
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Foods {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values: Vec<Food> = Vec::deserialize(deserializer)?;
        let mut foods = Foods::default();
        foods.extend(values.into_iter());
        Ok(foods)
    }
}

impl Foods {
    pub fn iter(&self) -> impl Iterator<Item = &Food> {
        self.values.values()
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn extend(&mut self, foods: impl Iterator<Item = Food>) {
        for food in foods {
            self.insert(food);
        }
    }

    pub fn insert(&mut self, food: Food) {
        self.values.insert(food.pos, food);
    }

    pub fn has_pos(&self, pos: Pos) -> bool {
        self.values.contains_key(&pos)
    }

    pub fn remove_with_pos(&mut self, pos: Pos) {
        self.values.remove(&pos);
    }

    pub fn boundaries(&self) -> Option<Boundaries> {
        let foods = self.values.keys();
        Boundaries::from_iterators(
            foods.clone().cloned().map(Pos::x),
            foods.cloned().map(Pos::y),
        )
    }

    pub fn empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct Sections {
    sections: Vec<Section>,
}

// packs sequence of directions to sequence of bytes
//
// each Direction is encoded using 2 bits because there are 4 values
// 4 directions can be encoded using 1 byte
//
// since last partition of directions can contain 1 to 4 values
// serializer pads such byte with zeroes
// deserializer requires to know how many directions to decode the last byte
//
fn pack_values(values: &[Direction]) -> Vec<u8> {
    let mut result = Vec::with_capacity((values.len() + 3) / 4);

    for chunk in values.chunks(4) {
        // start with empty byte
        let mut byte = 0u8;

        for dir in chunk {
            // move to left, leaving 2 bits of space
            byte <<= 2;
            // use bit OR to append 2 bit value to the end
            byte |= dir.encode();
        }

        // pad zeroes when chunk length is less than 4
        byte <<= 2 * (4 - chunk.len());

        result.push(byte);
    }

    result
}

fn unpack_values(bytes: &[u8], values_in_last_byte: u8) -> Vec<Direction> {
    let mut result = Vec::with_capacity(bytes.len() * 4);

    fn decode_byte(mut byte: u8, contains: u8) -> Vec<Direction> {
        let mut result = vec![];

        assert!(contains >= 1);
        assert!(contains <= 4);

        for i in 0..contains {
            let mask_shift = 6 - (2 * i);

            let mask = 0b11 << mask_shift;

            // extract bits using:
            // shifted 0b11 with & (removing bits to the left and right of the mask)
            // and then bit shift to the right to mask shift size
            // leaving you with a byte not exceeding decimal value 4 (2 bits)
            let dir_encoded = (byte & mask) >> mask_shift;
            result.push(Direction::decode(dir_encoded).unwrap()); // TODO handle unwrap
        }

        result
    }

    for (i, byte) in bytes.into_iter().enumerate() {
        let contains = if i == bytes.len() - 1 {
            values_in_last_byte
        } else {
            4u8
        };

        result.extend(decode_byte(*byte, contains));
    }

    result
}

#[test]
fn test_pack_values() {
    assert_eq!(
        pack_values(&[Direction::Up, Direction::Right, Direction::Bottom]),
        vec![0b00_11_01_00]
    )
}

#[test]
fn test_unpack_values() {
    assert_eq!(
        unpack_values(&vec![0b00_11_01_00], 3),
        vec![Direction::Up, Direction::Right, Direction::Bottom]
    )
}

#[test]
fn test_serde_sections() {
    let dirs_1 = vec![Direction::Up, Direction::Up, Direction::Up];

    let dirs_2 = vec![Direction::Up, Direction::Right, Direction::Bottom];

    for dirs in [dirs_1, dirs_2] {
        println!("Original dirs: {dirs:?}");
        let sections = Sections::from_directions(Pos::new(0, 0), dirs);

        let ser = serde_json::to_string(&sections).unwrap();
        dbg!(format!("{ser:?}"));
        let de = serde_json::from_str::<Sections>(&ser).unwrap();
        dbg!(&sections);
        dbg!(&de);

        assert_eq!(sections, de);
    }
}

// efficiently serialize Sections struct
// binary package structure:
//  - 4 and 4 bytes for X and Y dimensions of the beginning of the first section respectively
//  - 1 byte designated for the number of directions to decode in the last byte (see pack_values for more)
//  - the rest are packed directions
impl Serialize for Sections {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut bytes: Vec<u8> = vec![];

        // TODO handle unwrap
        let point = self.sections.first().unwrap().start();

        let x_bytes = point.x().to_be_bytes();
        let y_bytes = point.y().to_be_bytes();

        bytes.extend(x_bytes);
        bytes.extend(y_bytes);

        let dirs = self.iter_directions().collect::<Vec<_>>();

        let values_in_last_byte = {
            let rem = (dirs.len() % 4) as u8;
            if rem == 0 {
                4
            } else {
                rem
            }
        };

        bytes.push(values_in_last_byte);

        let packed = pack_values(dirs.as_ref());

        bytes.extend(packed);

        serializer.serialize_bytes(&bytes)
    }
}

// deserialize according to payload structure
// recreate Sections from deserialized values
impl<'de> Deserialize<'de> for Sections {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Sections;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid sequence of bytes")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let x = i32::from_be_bytes([
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(0, &"Not enough bytes"))?,
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(1, &"Not enough bytes"))?,
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2, &"Not enough bytes"))?,
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3, &"Not enough bytes"))?,
                ]);

                let y = i32::from_be_bytes([
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(4, &"Not enough bytes"))?,
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(5, &"Not enough bytes"))?,
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(6, &"Not enough bytes"))?,
                    seq.next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(7, &"Not enough bytes"))?,
                ]);

                let values_in_last_byte: u8 = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

                let mut bytes = vec![];

                // TODO handle it better
                while let Ok(Some(value)) = seq.next_element::<u8>() {
                    bytes.push(value);
                }

                let directions = unpack_values(&bytes, values_in_last_byte);

                Ok(Sections::from_directions(Pos::new(x, y), directions))
            }
        }

        let visitor = Visitor;

        deserializer.deserialize_bytes(visitor)
    }
}

impl Sections {
    // iter directions starting from the start of the first section
    pub fn iter_directions(&self) -> impl Iterator<Item = Direction> + '_ {
        self.sections.iter().map(|section| {
            let diff = section.end() - section.start();

            match (diff.x(), diff.y()) {
                (1, 0) => Direction::Right,
                (-1, 0) => Direction::Left,
                (0, 1) => Direction::Bottom,
                (0, -1) => Direction::Up,
                _ => unreachable!(),
            }
        })
    }

    pub fn iter_vertices(&self) -> impl Iterator<Item = Pos> + '_ {
        self.sections
            .iter()
            .map(|section| section.start())
            .chain(std::iter::once(self.head().end()))
    }

    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    pub fn head(&self) -> Section {
        self.as_ref().last().unwrap().clone()
    }

    pub fn tail(&self) -> Section {
        self.as_ref().first().unwrap().clone()
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
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

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, Eq, Hash)]
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

impl std::ops::Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
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

    pub fn boundaries_in_radius(self, x_radius: i32, y_radius: i32) -> Boundaries {
        Boundaries {
            min: Pos::new(self.x - x_radius, self.y - y_radius),
            max: Pos::new(self.x + x_radius, self.y + y_radius),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Bottom,
    Left,
    Right,
}

impl Direction {
    pub fn encode(&self) -> u8 {
        match self {
            Self::Up => 0b00,
            Self::Bottom => 0b01,
            Self::Left => 0b10,
            Self::Right => 0b11,
        }
    }

    pub fn decode(value: u8) -> Option<Self> {
        match value {
            0b00 => Some(Self::Up),
            0b01 => Some(Self::Bottom),
            0b10 => Some(Self::Left),
            0b11 => Some(Self::Right),
            _ => None,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Bottom,
            Self::Bottom => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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

    pub fn width(&self) -> u32 {
        (self.right_top() - self.left_top()).x as u32
    }

    pub fn height(&self) -> u32 {
        (self.right_bottom() - self.right_top()).y as u32
    }
}

#[derive(strum::EnumIs)]
pub enum RelationToBoundaries {
    Inside,
    Touching,
    Outside,
}

pub mod figures {

    #[derive(strum::EnumIter)]
    pub enum Figures {
        Diagonal2F,
        Diagonal3,
        X,
    }

    impl Figures {
        pub fn to_iter(&self) -> Vec<Vec<FigureCell>> {
            use matrix_to_iter as mi;

            let matrix = match self {
                Self::Diagonal2F => mi(diagonal_2f()),
                Self::Diagonal3 => mi(diagonal_3()),
                Self::X => mi(figure_x()),
            };
            matrix
        }

        pub fn x_dim(&self) -> usize {
            let matrix = match self {
                Self::Diagonal2F => x_dim(diagonal_2f),
                Self::Diagonal3 => x_dim(diagonal_3),
                Self::X => x_dim(figure_x),
            };
            matrix
        }

        pub fn y_dim(&self) -> usize {
            let matrix = match self {
                Self::Diagonal2F => y_dim(diagonal_2f),
                Self::Diagonal3 => y_dim(diagonal_3),
                Self::X => y_dim(figure_x),
            };
            matrix
        }
    }

    fn diagonal_2f() -> [[FigureCell; 2]; 2] {
        use FigureCell::*;

        [
            //
            [Empty, Food],
            [Food, Empty],
        ]
    }

    fn diagonal_3() -> [[FigureCell; 3]; 3] {
        use FigureCell::*;

        [
            [Food, Empty, Empty],
            [Empty, Food, Empty],
            [Empty, Empty, Food],
        ]
    }

    fn figure_x() -> [[FigureCell; 3]; 3] {
        use FigureCell::*;

        [
            [Food, Empty, Food],
            [Empty, Food, Empty],
            [Food, Empty, Food],
        ]
    }

    #[derive(Clone, strum::EnumIs)]
    pub enum FigureCell {
        Empty,
        Food,
    }

    fn x_dim<T, const C: usize, const R: usize, F: Fn() -> [[T; C]; R]>(f: F) -> usize {
        C
    }

    fn y_dim<T, const C: usize, const R: usize, F: Fn() -> [[T; C]; R]>(f: F) -> usize {
        R
    }

    fn matrix_to_iter<T, U, I>(array: T) -> Vec<Vec<I>>
    where
        T: IntoIterator<Item = U>,
        U: IntoIterator<Item = I>,
    {
        array
            .into_iter()
            .map(|inner_array| inner_array.into_iter().collect())
            .collect()
    }
}
