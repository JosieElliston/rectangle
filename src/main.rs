use egui::{
    Color32,
    ahash::{HashMap, HashMapExt, HashSet, HashSetExt},
};
use itertools::Itertools;
use rand::prelude::*;
use std::iter::once;

// TODO: rename to Color?
/// sides related by ! are opposite,
/// rather than by -, so that we can 0 index.
/// lives in -dim..=dim-1
/// eg for dim=3, it would be in [-3, -2, -1, 0, 1, 2]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct Side(i16);
impl Side {
    fn new(side: i16) -> Self {
        debug_assert!(
            side >= -(App::MAX_DIM as i16) && side < App::MAX_DIM as i16,
            "side should be in -{}..={}",
            App::MAX_DIM,
            App::MAX_DIM - 1
        );
        Side(side)
    }

    /// kinda like abs
    fn axis(self) -> Axis {
        Axis(Side(self.0.max(!self.0)))
    }

    /// checked conversion into axis
    fn into_axis(self) -> Axis {
        debug_assert!(self.0 >= 0, "cannot convert negative side into axis");
        Axis(self)
    }

    /// checked conversion into usize
    fn into_usize(self) -> usize {
        debug_assert!(self.0 >= 0, "cannot convert negative side into usize");
        self.0 as usize
    }

    /// maybe the should be called nonnegative,
    /// but it's still true that Side(0) refers to the positive direction of axis 0
    fn is_positive(self) -> bool {
        self.0 >= 0
    }

    fn get<'a, T>(self, pos: &'a [T], neg: &'a [T]) -> &'a T {
        if self.is_positive() {
            &pos[self.0 as usize]
        } else {
            &neg[!self.0 as usize]
        }
    }

    fn get_mut<'a, T>(self, pos: &'a mut [T], neg: &'a mut [T]) -> &'a mut T {
        if self.is_positive() {
            &mut pos[self.0 as usize]
        } else {
            &mut neg[!self.0 as usize]
        }
    }

    fn name(self) -> char {
        const POS_NAMES: &[char] = &['R', 'U', 'F', 'O', 'A', 'Γ', 'Θ', 'Ξ', 'Σ', 'Ψ'];
        const NEG_NAMES: &[char] = &['L', 'D', 'B', 'I', 'P', 'Δ', 'Λ', 'Π', 'Φ', 'Ω'];
        *self.get(POS_NAMES, NEG_NAMES)
    }

    const POS_KEYS: &[char] = &['f', 'e', 'r', 't', 'v', 'y', 'n', 'q', ',', '/'];
    const NEG_KEYS: &[char] = &['s', 'd', 'w', 'g', 'c', 'h', 'b', 'a', 'm', '.'];
    fn side_key(self) -> char {
        *self.get(Self::POS_KEYS, Self::NEG_KEYS)
    }
    fn try_from_key(dim: u16, key: egui::Key) -> Option<Self> {
        let key = key.symbol_or_name();
        if key.len() != 1 {
            return None;
        }
        let key = key.to_lowercase().chars().next().unwrap();
        Self::POS_KEYS[..dim as usize]
            .iter()
            .position(|&k| k == key)
            .map(|i| Self(i as i16))
            .or_else(|| {
                Self::NEG_KEYS[..dim as usize]
                    .iter()
                    .position(|&k| k == key)
                    .map(|i| Self(!(i as i16)))
            })
    }

    fn color(self) -> Color32 {
        let pos_colors: &[Color32] = &[
            Color32::from_rgb(255, 0, 0),
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(0, 255, 0),
            Color32::from_rgb(255, 0, 255),
            Color32::from_rgb(10, 170, 133),
            Color32::from_rgb(119, 72, 17),
            Color32::from_rgb(244, 159, 239),
            Color32::from_rgb(178, 152, 103),
            Color32::from_rgb(156, 245, 66),
            Color32::from_rgb(7, 133, 23),
        ];
        let neg_colors: &[Color32] = &[
            Color32::from_rgb(255, 128, 0),
            Color32::from_rgb(255, 255, 0),
            Color32::from_rgb(0, 128, 255),
            Color32::from_rgb(143, 16, 234),
            Color32::from_rgb(125, 170, 10),
            Color32::from_rgb(109, 69, 100),
            Color32::from_rgb(212, 169, 78),
            Color32::from_rgb(178, 121, 103),
            Color32::from_rgb(66, 212, 245),
            Color32::from_rgb(47, 47, 189),
        ];
        *self.get(pos_colors, neg_colors)
    }
}
impl std::ops::Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        Side(!self.0)
    }
}

/// like Side, but only for the positive directions
/// lives in 0..=dim-1
/// eg for dim=3, it would be in [0, 1, 2]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct Axis(Side);
impl Axis {
    fn new(axis: i16) -> Self {
        debug_assert!(
            axis < App::MAX_DIM as _,
            "axis should be less than {}",
            App::MAX_DIM
        );
        Axis(Side(axis))
    }

    fn into_side(self) -> Side {
        self.0
    }

    fn into_usize(self) -> usize {
        self.0.0 as usize
    }

    fn from_usize(axis: usize) -> Self {
        Self::new(axis as i16)
    }

    const AXIS_KEYS: &[char] = &['k', 'j', 'l', 'i', 'u', 'o', 'p', ';', '[', '\''];
    fn axis_key(self) -> char {
        Self::AXIS_KEYS[self.into_usize()]
    }
    fn try_from_key(dim: u16, key: egui::Key) -> Option<Self> {
        let key = key.symbol_or_name();
        if key.len() != 1 {
            return None;
        }
        let key = key.to_lowercase().chars().next().unwrap();
        Self::AXIS_KEYS[..dim as usize]
            .iter()
            .position(|&k| k == key)
            .map(Self::from_usize)
    }
}
// impl From<Axis> for Side {
//     fn from(axis: Axis) -> Self {
//         axis.0
//     }
// }
// impl From<Axis> for usize {
//     fn from(axis: Axis) -> Self {
//         axis.0.0 as usize
//     }
// }

/// lives in ±n and -n+1 to n-1 every other
/// eg for n=3, it would be in [-3, -2, 0, 2, 3]
/// eg for n=4, it would be in [-4, -3, -1, 1, 3, 4]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct Coord(i16);
impl std::ops::Neg for Coord {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Coord(-self.0)
    }
}

// TODO: rename to layer?
/// A shape is a \[Cut], so a 2x3x4 would be a \[Cut(2), Cut(3), Cut(4)]
/// lives in [1, 2, 3, ...]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cut(i16);
impl Cut {
    /// all possible coords along this axis
    fn coords(self) -> impl Iterator<Item = Coord> {
        once(-self.0)
            .chain((1 - self.0..self.0).step_by(2))
            .chain(once(self.0))
            .map(Coord)
    }
}

// #[derive(Clone, Debug, PartialEq, Eq)]
// struct Shape(Vec<Cut>);

/// any (possibly internal) position
/// at most one of the coords is ±n
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Position(Vec<Coord>);
impl Position {
    fn try_new(shape: &[Cut], coords: Vec<Coord>) -> Option<Self> {
        if coords
            .iter()
            .zip(shape)
            .filter(|(coord, cut)| coord.0.abs() == cut.0)
            .count()
            <= 1
        {
            Some(Self(coords))
        } else {
            None
        }
    }

    fn new(shape: &[Cut], coords: Vec<Coord>) -> Self {
        if cfg!(debug_assertions) {
            Self::try_new(shape, coords).unwrap()
        } else {
            Self(coords)
        }
    }

    /// gives all positions for this shape, including internal ones
    fn all(shape: &[Cut]) -> impl Iterator<Item = Self> {
        shape
            .iter()
            .map(|cut| cut.coords().collect::<Vec<_>>())
            .multi_cartesian_product()
            .filter_map(|coords| Position::try_new(shape, coords))
    }

    // /// at most one of the coords is ±n
    // fn is_valid(&self, shape: &[Cut]) -> bool {
    //     self.0
    //         .iter()
    //         .zip(shape)
    //         .filter(|(coord, cut)| coord.0.abs() == cut.0)
    //         .count()
    //         <= 1
    // }

    /// exactly one of the coords is ±n
    fn is_sticker(&self, shape: &[Cut]) -> bool {
        self.0
            .iter()
            .zip(shape)
            .filter(|(coord, cut)| coord.0.abs() == cut.0)
            .count()
            == 1
    }

    /// at least one of the coords is ±(n-1)
    /// none of the coords are ±n
    fn is_piece(&self, shape: &[Cut]) -> bool {
        self.0
            .iter()
            .zip(shape)
            .filter(|(coord, cut)| coord.0.abs() == cut.0 - 1)
            .count()
            > 0
            && self
                .0
                .iter()
                .zip(shape)
                .filter(|(coord, cut)| coord.0.abs() == cut.0)
                .count()
                == 0
    }
}
// impl From<Sticker> for Position {
//     fn from(sticker: Sticker) -> Self {
//         sticker.0
//     }
// }
// impl From<Piece> for Position {
//     fn from(piece: Piece) -> Self {
//         piece.0
//     }
// }

/// exactly one of the coords is ±n
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Sticker(Position);
impl Sticker {
    fn try_from(shape: &[Cut], pos: Position) -> Option<Self> {
        pos.is_sticker(shape).then_some(Self(pos))
    }

    fn try_new(shape: &[Cut], coords: Vec<Coord>) -> Option<Self> {
        Position::try_new(shape, coords).and_then(|pos| Self::try_from(shape, pos))
    }

    fn new(shape: &[Cut], coords: Vec<Coord>) -> Self {
        if cfg!(debug_assertions) {
            Self::try_new(shape, coords).unwrap()
        } else {
            Self(Position(coords))
        }
    }

    /// the piece this sticker lies on
    fn piece(&self, shape: &[Cut]) -> Piece {
        Piece::new(
            shape,
            self.0
                .0
                .iter()
                .zip(shape)
                .map(|(coord, cut)| {
                    if coord.0 == cut.0 {
                        Coord(coord.0 - 1)
                    } else if -coord.0 == cut.0 {
                        Coord(coord.0 + 1)
                    } else {
                        *coord
                    }
                })
                .collect(),
        )
    }

    // TODO: refactor to use this function
    /// which side this sticker lies on
    fn side(&self, shape: &[Cut]) -> Side {
        self.0
            .0
            .iter()
            .zip(shape)
            .enumerate()
            .find_map(|(i, (coord, cut))| {
                if coord.0 == cut.0 {
                    Some(Side::new(i as i16))
                } else if -coord.0 == cut.0 {
                    Some(Side::new(!(i as i16)))
                } else {
                    None
                }
            })
            .expect("sticker should be on a side")
    }
}

/// at least one of the coords is ±(n-1)
/// none of the coords are ±n
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Piece(Position);
impl Piece {
    fn try_from(shape: &[Cut], pos: Position) -> Option<Self> {
        pos.is_piece(shape).then_some(Self(pos))
    }

    fn try_new(shape: &[Cut], coords: Vec<Coord>) -> Option<Self> {
        Position::try_new(shape, coords).and_then(|pos| Self::try_from(shape, pos))
    }

    fn new(shape: &[Cut], coords: Vec<Coord>) -> Self {
        if cfg!(debug_assertions) {
            Self::try_new(shape, coords).unwrap()
        } else {
            Self(Position(coords))
        }
    }

    /// which sides this sticker lies on
    fn sides(&self, shape: &[Cut]) -> impl Iterator<Item = Side> {
        let mut ret = self
            .0
            .0
            .iter()
            .zip(shape)
            .enumerate()
            .filter_map(|(i, (coord, cut))| {
                if coord.0 + 1 == cut.0 {
                    Some(Side::new(i as i16))
                } else if 1 - coord.0 == cut.0 {
                    Some(Side::new(!(i as i16)))
                } else {
                    None
                }
            })
            .peekable();
        debug_assert!(ret.peek().is_some(), "piece should be on at least one side");
        ret
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LayerMask(Vec<bool>);
impl LayerMask {
    // fn new(n: i16) -> Self {
    //     let mut ret = vec![false; (n as usize - 1) / 2];
    //     if n > 1 {
    //         ret[0] = true;
    //     }
    //     LayerMask(ret)
    // }
    fn new() -> Self {
        let mut ret = vec![false; (App::MAX_LAYERS as usize - 1) / 2];
        ret[0] = true;
        LayerMask(ret)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SideTurn {
    layers: LayerMask,
    side: Side,
    from: Axis,
    to: Axis,
}
impl SideTurn {
    fn inverse(&self) -> Self {
        SideTurn {
            layers: self.layers.clone(),
            side: self.side,
            from: self.to,
            to: self.from,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PuzzleTurn {
    from: Axis,
    to: Axis,
}
impl PuzzleTurn {
    fn inverse(&self) -> Self {
        PuzzleTurn {
            from: self.to,
            to: self.from,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Turn {
    Side(SideTurn),
    Puzzle(PuzzleTurn),
}
impl Turn {
    fn inverse(&self) -> Self {
        match self {
            Turn::Side(side_turn) => Self::Side(side_turn.inverse()),
            Turn::Puzzle(puzzle_turn) => Self::Puzzle(puzzle_turn.inverse()),
        }
    }
}

// TODO: don't store shape here
#[derive(Clone, Debug)]
enum TurnBuilder {
    Side {
        shape: Box<[Cut]>,
        layers: LayerMask,
        side: Option<Side>,
        from: Option<Axis>,
    },
    Puzzle {
        shape: Box<[Cut]>,
        from: Option<Axis>,
    },
}
impl TurnBuilder {
    fn new(shape: &[Cut]) -> Self {
        TurnBuilder::Side {
            shape: shape.into(),
            layers: LayerMask::new(),
            side: None,
            from: None,
        }
    }

    /// returns Some if the turn is complete
    #[inline(never)]
    fn update(&mut self, key: egui::Key) -> Option<Turn> {
        if key == egui::Key::Escape {
            *self = TurnBuilder::new(self.shape());
            return None;
        }
        if key == egui::Key::X {
            *self = TurnBuilder::Puzzle {
                shape: self.shape().into(),
                from: None,
            };
            return None;
        }
        if let Ok(key) = key.name().parse::<usize>() {
            if key == 0 {
                return None;
            }
            // if key > (self.n() as usize - 1) / 2 {
            //     return None;
            // }
            if let TurnBuilder::Puzzle { shape, .. } = self {
                *self = TurnBuilder::Side {
                    shape: shape.clone(),
                    layers: LayerMask::new(),
                    side: None,
                    from: None,
                };
            }
            match self {
                TurnBuilder::Side { layers, .. } => {
                    layers.0[key - 1] = !layers.0[key - 1];
                }
                TurnBuilder::Puzzle { .. } => unreachable!(),
            }
            return None;
        }
        if let Some(s) = Side::try_from_key(self.shape().len() as _, key) {
            match self {
                TurnBuilder::Side { side, from, .. } => {
                    *side = Some(s);
                    *from = None;
                }
                TurnBuilder::Puzzle { shape, .. } => {
                    *self = TurnBuilder::Side {
                        shape: shape.clone(),
                        layers: LayerMask::new(),
                        side: Some(s),
                        from: None,
                    };
                }
            }
            return None;
        }
        match self {
            TurnBuilder::Side {
                shape,
                side,
                from,
                layers,
                ..
            } => {
                if let Some(s) = side {
                    if let Some(f) = from {
                        if let Some(t) = Axis::try_from_key(shape.len() as _, key) {
                            let ret = Some(Turn::Side(SideTurn {
                                side: *s,
                                from: *f,
                                to: t,
                                layers: layers.clone(),
                            }));
                            *from = None;
                            return ret;
                        }
                    } else {
                        *from = Axis::try_from_key(shape.len() as _, key);
                    }
                }
            }
            TurnBuilder::Puzzle { shape, from, .. } => {
                if let Some(f) = from {
                    if let Some(t) = Axis::try_from_key(shape.len() as _, key) {
                        let ret = Some(Turn::Puzzle(PuzzleTurn { from: *f, to: t }));
                        *from = None;
                        return ret;
                    }
                } else {
                    *from = Axis::try_from_key(shape.len() as _, key);
                }
            }
        }
        None
    }

    // TODO: these are used for evil
    fn shape(&self) -> &[Cut] {
        match self {
            TurnBuilder::Side { shape, .. } => shape,
            TurnBuilder::Puzzle { shape, .. } => shape,
        }
    }
}

#[derive(Clone, Debug)]
enum TurnError {
    /// `from` and `to` don't define a plane of rotation
    UndefinedPlane,
    // Blocked, // for bandaging
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Puzzle {
    shape: Vec<Cut>,
    // #[serde(with = "serde_map")]
    stickers: HashMap<Sticker, Side>,
}
impl Puzzle {
    #[inline(never)]
    fn new(shape: &[Cut]) -> Self {
        Puzzle {
            shape: shape.to_vec(),
            stickers: Position::all(shape)
                .filter_map(|pos| {
                    Sticker::try_from(shape, pos).map(|sticker| {
                        let side = sticker.side(shape);
                        (sticker, side)
                    })
                })
                .collect(),
        }
    }

    #[inline(never)]
    fn is_solved(&self) -> bool {
        let mut pos_colors: Vec<Option<Side>> = vec![None; self.shape.len()];
        let mut neg_colors: Vec<Option<Side>> = vec![None; self.shape.len()];
        for (sticker, color) in &self.stickers {
            let entry = sticker
                .side(&self.shape)
                .get_mut(&mut pos_colors, &mut neg_colors);
            if let Some(old_color) = entry
                && old_color != color
            {
                return false;
            }
            *entry = Some(*color);
        }
        true
    }

    // #[inline(never)]
    // fn turn_side(&mut self, turn: &SideTurn) -> Result<(), TurnError> {
    //     let SideTurn {
    //         ref layers,
    //         side,
    //         from,
    //         to,
    //     } = *turn;
    //     if side == from.into_side()
    //         || !side == from.into_side()
    //         || side == to.into_side()
    //         || !side == to.into_side()
    //         || from == to
    //     {
    //         return Err(TurnError::UndefinedPlane);
    //     }
    //     // assert!(from.0 >= 0 && to.0 >= 0);
    //     // TODO: i don't think this needs to be a hashmap
    //     let mut new_stickers = Vec::new();
    //     let mut from_pos = vec![Coord(0); self.shape.len()].into_boxed_slice();
    //     for pos in self.stickers.keys() {
    //         // TODO: layer mask
    //         // if if side.0 >= 0 {
    //         //     layers.0[pos[side.0 as usize].0 as usize]
    //         // } else {
    //         //     layers.0[pos[(!side.0) as usize].0 as usize]
    //         // } {
    //         if if side.is_positive() {
    //             ((self.shape[side.into_usize()].0 - 1)..=self.shape[side.into_usize()].0)
    //                 .contains(&pos[side.into_usize()].0)
    //         } else {
    //             ((-self.shape[(!side).into_usize()].0)..=(1 - self.shape[(!side).into_usize()].0))
    //                 .contains(&pos[(!side).into_usize()].0)
    //         } {
    //             // TODO: compute to_pos instead of from_pos???
    //             // let mut from_pos = pos.clone();
    //             // this is actually faster, i checked
    //             from_pos.clone_from_slice(pos);

    //             // for cuboids, if you can't turn 90 degrees, just turn 180 degrees
    //             if self.shape[from.into_usize()] == self.shape[to.into_usize()] {
    //                 from_pos[from.into_usize()] = pos[to.into_usize()];
    //                 from_pos[to.into_usize()] = -pos[from.into_usize()];
    //             } else {
    //                 from_pos[from.into_usize()] = -pos[from.into_usize()];
    //                 from_pos[to.into_usize()] = -pos[to.into_usize()];
    //             }
    //             new_stickers.push((pos.clone(), self.stickers[&from_pos]));
    //         }
    //     }
    //     self.stickers.extend(new_stickers);
    //     Ok(())
    // }
    #[inline(never)]
    fn turn_side(&mut self, turn: &SideTurn) -> Result<(), TurnError> {
        let SideTurn {
            ref layers,
            side,
            from,
            to,
        } = *turn;
        if side == from.into_side()
            || !side == from.into_side()
            || side == to.into_side()
            || !side == to.into_side()
            || from == to
        {
            return Err(TurnError::UndefinedPlane);
        }
        let mut new_stickers = Vec::new();
        let mut from_pos = Sticker(Position(vec![Coord(0); self.shape.len()]));
        for (pos, old_color) in &self.stickers {
            // TODO: layer mask
            // if if side.0 >= 0 {
            //     layers.0[pos[side.0 as usize].0 as usize]
            // } else {
            //     layers.0[pos[(!side.0) as usize].0 as usize]
            // } {
            // TODO: rename pos to sticker
            if if side.is_positive() {
                ((self.shape[side.into_usize()].0 - 1)..=self.shape[side.into_usize()].0)
                    .contains(&pos.0.0[side.into_usize()].0)
            } else {
                ((-self.shape[(!side).into_usize()].0)..=(1 - self.shape[(!side).into_usize()].0))
                    .contains(&pos.0.0[(!side).into_usize()].0)
            } {
                // TODO: compute to_pos instead of from_pos???

                // let mut from_pos = pos.clone();
                // this is actually faster, i checked
                from_pos.0.0.clone_from_slice(&pos.0.0);

                // for cuboids, if you can't turn 90 degrees, just turn 180 degrees
                if self.shape[from.into_usize()] == self.shape[to.into_usize()] {
                    from_pos.0.0[from.into_usize()] = pos.0.0[to.into_usize()];
                    from_pos.0.0[to.into_usize()] = -pos.0.0[from.into_usize()];
                } else {
                    from_pos.0.0[from.into_usize()] = -pos.0.0[from.into_usize()];
                    from_pos.0.0[to.into_usize()] = -pos.0.0[to.into_usize()];
                }
                new_stickers.push((old_color as *const _, self.stickers[&from_pos]));
            }
        }
        for (old_sticker, new_sticker) in new_stickers {
            // # Safety: old sticker is where the entry was,
            // and we didn't modify the map so it should still be there.
            unsafe {
                *(old_sticker as *mut _) = new_sticker;
            }
        }
        Ok(())
    }

    #[inline(never)]
    fn turn_puzzle(&mut self, turn: &PuzzleTurn) -> Result<(), TurnError> {
        let PuzzleTurn { from, to } = *turn;
        if from == to {
            return Err(TurnError::UndefinedPlane);
        }
        let mut new_stickers = Vec::new();
        for pos in self.stickers.keys() {
            let mut from_pos = pos.clone();
            from_pos.0.0[from.into_usize()] = pos.0.0[to.into_usize()];
            from_pos.0.0[to.into_usize()] = -pos.0.0[from.into_usize()];
            new_stickers.push((pos.clone(), self.stickers[&from_pos]));
        }
        self.stickers = HashMap::from_iter(new_stickers);
        Ok(())
    }

    #[inline(never)]
    fn turn(&mut self, turn: &Turn) -> Result<(), TurnError> {
        match turn {
            Turn::Side(turn) => self.turn_side(turn),
            Turn::Puzzle(turn) => self.turn_puzzle(turn),
        }
    }

    #[inline(never)]
    fn scramble(&mut self, rng: &mut impl Rng) {
        const SCRAMBLE_N: usize = 1000;
        let start = std::time::Instant::now();
        let dim = self.shape.len() as i16;
        let side_dist = rand::distr::Uniform::new(-dim, dim).unwrap();
        let axis_dist = rand::distr::Uniform::new(0, dim).unwrap();
        for _ in 0..SCRAMBLE_N {
            // TODO: layer mask
            let side = rng.sample(side_dist);
            let from = rng.sample(axis_dist);
            if side == from || !side == from {
                continue;
            }
            let to = rng.sample(axis_dist);
            if side == to || !side == to || from == to {
                continue;
            }
            self.turn_side(&SideTurn {
                layers: LayerMask::new(),
                side: Side(side),
                from: Axis::new(from),
                to: Axis::new(to),
            })
            .unwrap();
        }
        println!("scrambled  in {:?}", start.elapsed());
    }
}

/// mapping from Pos to (x, y) coordinates
/// +x is right, +y is up
#[derive(Clone, Debug)]
struct Layout2dBuilder {
    width: usize,
    height: usize,
    mapping: Vec<(Vec<Coord>, (usize, usize))>,
}
impl Layout2dBuilder {
    #[inline(never)]
    fn new(shape: &[Cut]) -> Self {
        if shape.is_empty() {
            return Self {
                width: 1,
                height: 1,
                mapping: vec![(Vec::new(), (0, 0))],
            };
        }
        // TODO: refactor to use Position
        let horizontal = shape.len() % 2 == 1;
        let lower = Self::new(&shape[..shape.len() - 1]);
        let last = *shape.last().unwrap();
        assert_eq!(last.coords().count(), last.0 as usize + 2);
        let mut ret = Self {
            width: 0,
            height: 0,
            mapping: Vec::with_capacity(lower.mapping.len() * (last.0 as usize + 2)),
        };
        for (i, new_coord) in last.coords().enumerate() {
            let mut lower = lower.clone();
            lower.mapping = lower
                .mapping
                .into_iter()
                .map(|(mut pos, xy)| {
                    pos.push(new_coord);
                    (pos, xy)
                })
                .collect();
            if new_coord.0.abs() == last.0 {
                // only keep valid positions
                // we only need to check this if it's a cap
                lower.mapping.retain(|(pos, _xy)| {
                    pos.iter()
                        .zip(shape)
                        .filter(|(coord, cut)| coord.0.abs() == cut.0)
                        .count()
                        <= 1
                });
                // TODO: possibly shrink margins
            } else {
                debug_assert!(lower.mapping.iter().all(|(pos, _xy)| {
                    pos.iter()
                        .zip(shape)
                        .filter(|(coord, cut)| coord.0.abs() == cut.0)
                        .count()
                        <= 1
                }));
            }
            let shift = if horizontal {
                lower.width
            } else {
                lower.height
            };
            let shift = if shape.len() > 2 {
                (shift + 1) * i
            } else {
                shift * i
            };
            if horizontal {
                lower.right(shift);
            } else {
                lower.down(shift);
            }
            ret.union(lower);
        }
        assert!(ret.mapping.len() <= lower.mapping.len() * (last.0 as usize + 2));
        ret
    }

    #[inline(never)]
    fn union(&mut self, other: Self) {
        self.width = self.width.max(other.width);
        self.height = self.height.max(other.height);
        let self_len = self.mapping.len();
        let other_len = other.mapping.len();
        self.mapping.extend(other.mapping);
        assert_eq!(self.mapping.len(), self_len + other_len);
    }

    #[inline(never)]
    fn right(&mut self, shift: usize) {
        self.mapping.iter_mut().for_each(|(_pos, (x, _y))| {
            *x += shift;
        });
        self.width += shift;
    }

    #[inline(never)]
    fn down(&mut self, shift: usize) {
        self.mapping.iter_mut().for_each(|(_pos, (_x, y))| {
            *y += shift;
        });
        self.height += shift;
    }
}

#[derive(Clone, Debug)]
struct Layout2d {
    width: usize,
    height: usize,
    mapping: HashMap<Position, (usize, usize)>,
    inverse: HashMap<(usize, usize), Position>,
}
impl Layout2d {
    #[inline(never)]
    fn new(shape: &[Cut]) -> Self {
        let builder = Layout2dBuilder::new(shape);
        Layout2d {
            width: builder.width,
            height: builder.height,
            mapping: builder
                .mapping
                .clone()
                .into_iter()
                .map(|(pos, xy)| (Position::new(shape, pos), xy))
                .collect(),
            inverse: builder
                .mapping
                .into_iter()
                .map(|(pos, xy)| (xy, Position::new(shape, pos)))
                .collect(),
        }
    }
}

// #[derive(Clone, Debug)]
// enum Layout {
//     // OneD(Layout1d),
//     TwoD(Layout2d),
//     // ThreeD(Layout3d),
// }

#[derive(Clone, Debug)]
struct StickerFormatBuilder {
    outline_color: Option<Color32>,
    outline_width: Option<f32>,
    sticker_scale: Option<f32>,
    sticker_opacity: Option<f32>,
}
impl StickerFormatBuilder {
    pub const NONE: Self = StickerFormatBuilder {
        outline_color: None,
        outline_width: None,
        sticker_scale: None,
        sticker_opacity: None,
    };

    fn update(&mut self, other: &StickerFormatBuilder) {
        self.outline_color = self.outline_color.or(other.outline_color);
        self.outline_width = self.outline_width.or(other.outline_width);
        self.sticker_scale = self.sticker_scale.or(other.sticker_scale);
        self.sticker_opacity = self.sticker_opacity.or(other.sticker_opacity);
    }

    fn build_or(&self, default: &StickerFormat) -> StickerFormat {
        StickerFormat {
            outline_color: self.outline_color.unwrap_or(default.outline_color),
            outline_width: self.outline_width.unwrap_or(default.outline_width),
            sticker_scale: self.sticker_scale.unwrap_or(default.sticker_scale),
            sticker_opacity: self.sticker_opacity.unwrap_or(default.sticker_opacity),
        }
    }
}

// TODO: better name
#[derive(Clone, Debug)]
struct StickerFormat {
    outline_color: Color32,
    /// lives in [0.0, 1.0],
    /// where 1.0 is the size of a sticker
    outline_width: f32,
    /// lives in [0.0, 1.0]
    sticker_scale: f32,
    /// lives in [0.0, 1.0]
    sticker_opacity: f32,
}
// impl Default for StickerFormat {
//     fn default() -> Self {
//         StickerFormat {
//             outline_color: Color32::from_rgb(100, 100, 100),
//             outline_width: 0.02,
//             sticker_scale: 1.0,
//             sticker_opacity: 1.0,
//         }
//     }
// }

#[derive(Clone, Debug)]
struct FilterTerm {
    must_have: HashSet<Side>,
    cant_have: HashSet<Side>,
}
impl FilterTerm {
    fn matches(&self, shape: &[Cut], piece: &Piece) -> bool {
        for side in piece.sides(shape) {
            if !self.must_have.contains(&side) {
                return false;
            }
            if self.cant_have.contains(&side) {
                return false;
            }
        }
        true
    }
}

/// the filter matches a piece if it matches any of the terms
#[derive(Clone, Debug)]
struct Filter {
    terms: Vec<FilterTerm>,
    format: StickerFormatBuilder,
}
impl Filter {
    fn matches(&self, shape: &[Cut], piece: &Piece) -> bool {
        self.terms.iter().any(|term| term.matches(shape, piece))
    }
}

/// a entire filter stage is rendered at once,
/// with the last Some property of the filter being applied to that piece
#[derive(Clone, Debug)]
struct FilterStage(Vec<Filter>);
impl FilterStage {
    // fn try_get(&self, shape: &[Cut], piece: &[Coord]) -> Option<StickerFormatBuilder> {
    //     for filter in &self.0 {
    //         if let Some(format) = filter.try_get(shape, piece) {
    //             return Some(format);
    //         }
    //     }
    //     None
    // }
}

#[derive(Clone, Debug)]
struct FilterSequence(Vec<FilterStage>);
// impl FilterSequence {
//     fn new() -> Self {
//         FilterSequence(Vec::new())
//     }
// }

#[derive(Clone, Debug)]
struct App {
    puzzle: Puzzle,
    layout: Layout2d,
    /// where the labels for the sides go
    /// the centers if odd and offset in the positive direction if even
    side_positions: HashMap<Side, Piece>,
    turn_builder: TurnBuilder,
    clicked_pieces: HashSet<Piece>,
    internal_color: Color32,
    internal_format: StickerFormat,
    hovered_format: StickerFormatBuilder,
    clicked_format: StickerFormatBuilder,
    gripped_format: StickerFormatBuilder,
    default_filter_format: StickerFormat,
    default_no_filter_format: StickerFormat,
    filter_sequence: FilterSequence,
    filter_stage: Option<usize>,
}
impl App {
    const MAX_DIM: usize = 10;
    const MAX_LAYERS: i16 = 19;

    #[inline(never)]
    fn new(shape: &[Cut]) -> Self {
        assert!(!shape.is_empty(), "dimension should be greater than 0");
        assert!(
            shape.len() <= Self::MAX_DIM,
            "dimension should be less than or equal to {}",
            Self::MAX_DIM
        );
        assert!(
            shape.iter().all(|cut| cut.0 > 0),
            "side should be greater than 0"
        );
        assert!(
            shape.iter().all(|cut| cut.0 <= Self::MAX_LAYERS),
            "side should be less than or equal to {}",
            Self::MAX_LAYERS
        );
        // println!("{:?}", Position::all(shape).collect::<Vec<_>>());
        // panic!();

        let start = std::time::Instant::now();
        let puzzle = Puzzle::new(shape);
        println!("puzzle gen in {:?}", start.elapsed());

        let start = std::time::Instant::now();
        // let layout = Layout::TwoD(Layout2d::new(shape));
        let layout = Layout2d::new(shape);
        println!("layout gen in {:?}", start.elapsed());
        // if let Layout::TwoD(layout) = &layout {
        //     for (pos, xy) in layout.mapping.iter() {
        //         println!(
        //             "{:?} -> {:?}",
        //             pos.iter().map(|c| c.0).collect::<Vec<_>>(),
        //             xy,
        //         );
        //     }
        // } else {
        //     unreachable!()
        // }

        #[inline(never)]
        fn get_side_positions(shape: &[Cut]) -> HashMap<Side, Piece> {
            let mut side_positions = HashMap::new();
            for (axis, cut) in shape.iter().enumerate() {
                {
                    // positive
                    let mut pos = shape
                        .iter()
                        .map(|cut| if cut.0 % 2 == 1 { Coord(0) } else { Coord(1) })
                        .collect::<Vec<_>>();
                    pos[axis] = Coord(cut.0 - 1);
                    side_positions.insert(Side(axis as i16), Piece::new(shape, pos));
                }
                {
                    // negative
                    let mut pos = shape
                        .iter()
                        .map(|cut| if cut.0 % 2 == 1 { Coord(0) } else { Coord(1) })
                        .collect::<Vec<_>>();
                    pos[axis] = Coord(1 - cut.0);
                    side_positions.insert(Side(!(axis as i16)), Piece::new(shape, pos));
                }
            }
            side_positions
        }
        // println!("width: {}, height: {}", layout.width, layout.height);
        // for (pos, xy) in &layout.mapping {
        //     println!(
        //         "{:?} -> {:?}",
        //         pos.iter().map(|c| c.0).collect::<Vec<_>>(),
        //         xy,
        //     );
        // }
        App {
            puzzle,
            layout,
            side_positions: get_side_positions(shape),
            turn_builder: TurnBuilder::new(shape),
            clicked_pieces: HashSet::new(),
            internal_color: Color32::DARK_GRAY,
            internal_format: StickerFormat {
                outline_color: Color32::BLACK,
                outline_width: 0.05,
                sticker_scale: 1.0,
                sticker_opacity: 1.0,
            },
            hovered_format: StickerFormatBuilder {
                outline_color: Some(Color32::WHITE),
                outline_width: Some(0.1),
                sticker_scale: None,
                sticker_opacity: None,
            },
            clicked_format: StickerFormatBuilder {
                outline_color: Some(Color32::LIGHT_GRAY),
                outline_width: Some(0.1),
                sticker_scale: None,
                sticker_opacity: None,
            },
            gripped_format: StickerFormatBuilder {
                outline_color: Some(Color32::GRAY),
                outline_width: Some(0.05),
                sticker_scale: None,
                sticker_opacity: None,
            },
            default_filter_format: StickerFormat {
                outline_color: Color32::BLACK,
                outline_width: 0.05,
                sticker_scale: 0.7,
                sticker_opacity: 0.5,
            },
            default_no_filter_format: StickerFormat {
                outline_color: Color32::BLACK,
                outline_width: 0.05,
                sticker_scale: 1.0,
                sticker_opacity: 1.0,
            },
            filter_sequence: FilterSequence(Vec::new()),
            filter_stage: None,
        }
    }

    // #[inline(never)]
    // fn render_png(&self, path: &str) {
    //     // let Layout::TwoD(layout) = &self.layout else {
    //     //     panic!("render_png only works for Layout2d");
    //     // };
    //     let start = std::time::Instant::now();
    //     let mut buf = vec![0; self.layout.width * self.layout.height * 3];

    //     let mut draw_sticker = |pos: &Position, color: Color32| {
    //         let (x, y) = self.layout.mapping[pos];
    //         let i = ((self.layout.height - y - 1) * self.layout.width + x) * 3;
    //         buf[i] = color.r();
    //         buf[i + 1] = color.g();
    //         buf[i + 2] = color.b();
    //     };

    //     for pos in Position::all(&self.puzzle.shape) {
    //         draw_sticker(&pos, Color32::GRAY);
    //     }
    //     for (pos, side) in &self.puzzle.stickers {
    //         draw_sticker(&pos.0, side.color());
    //     }
    //     println!("buffer gen in {:?}", start.elapsed());

    //     let start = std::time::Instant::now();
    //     image::save_buffer(
    //         path,
    //         &buf,
    //         self.layout.width as _,
    //         self.layout.height as _,
    //         image::ColorType::Rgb8,
    //     )
    //     .unwrap();
    //     println!("image save in {:?}", start.elapsed());
    // }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ctx.input(|i| {
                    // handle input
                    for event in i.events.iter() {
                        if let egui::Event::Key {
                            key,
                            physical_key: _,
                            pressed,
                            repeat,
                            modifiers: _,
                        } = event
                        {
                            if *pressed && !repeat {
                                if let Some(turn) = self.turn_builder.update(*key) {
                                    self.puzzle.turn(&turn);
                                    println!("solved: {}", self.puzzle.is_solved());
                                }
                            }
                        }
                    }
                });

                // draw ui
                if ui.button("scramble").clicked() {
                    self.puzzle.scramble(&mut rand::rng());
                }

                // assert!(self.puzzle.shape.len() == 3);
                // fn project_3_2(coords: (f32, f32, f32)) -> (f32, f32) {
                //     (coords.0, coords.1)
                // }
                // fn pos_to_vertexes_3(
                //     pos: &[Coord]
                // ) -> Vec<(f32, f32, f32)>  {
                //     assert!(pos.len() == 3);
                // }

                // puzzle: &Puzzle, hovered: Option<&Piece>, clicked: &[&Piece], gripped: Option<Side>,

                // draw puzzle
                let rect = ui.available_rect_before_wrap();
                let scale = f32::min(
                    rect.width() / self.layout.width as f32,
                    rect.height() / self.layout.height as f32,
                );

                let screen_of_pos = |pos: &Position| -> egui::Pos2 {
                    let (x, y) = self.layout.mapping[pos];
                    egui::Pos2::new(0.5 + x as f32, 0.5 + (self.layout.height - 1 - y) as f32)
                        * scale
                };
                let pos_of_screen = |screen: egui::Pos2| -> Option<Position> {
                    let x = (screen.x / scale - 0.5).round() as i32;
                    let y = self.layout.height as i32 - 1 - (screen.y / scale - 0.5).round() as i32;
                    if !(0..self.layout.width as _).contains(&x)
                        || !(0..self.layout.height as _).contains(&y)
                    {
                        return None;
                    }
                    self.layout.inverse.get(&(x as usize, y as usize)).cloned()
                };

                let hovered_pos: Option<Position> =
                    ui.input(|i| i.pointer.hover_pos()).and_then(pos_of_screen);
                let hovered_piece = hovered_pos.and_then(|pos| {
                    Piece::try_from(&self.puzzle.shape, pos.clone()).or_else(|| {
                        Sticker::try_from(&self.puzzle.shape, pos.clone())
                            .map(|sticker| sticker.piece(&self.puzzle.shape))
                    })
                });

                // if we clicked, added the hovered piece to the clicked pieces
                if let Some(hovered_piece) = &hovered_piece
                    && ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
                {
                    if self.clicked_pieces.contains(hovered_piece) {
                        self.clicked_pieces.remove(hovered_piece);
                    } else {
                        self.clicked_pieces.insert(hovered_piece.clone());
                    }
                }

                // TODO: layer mask
                let gripped_side = match &self.turn_builder {
                    TurnBuilder::Side { layers, side, .. } => *side,
                    TurnBuilder::Puzzle { .. } => None,
                };
                let format_sticker = |sticker: &Sticker| -> StickerFormat {
                    let mut ret = StickerFormatBuilder::NONE;
                    if let Some(hovered_piece) = hovered_piece.clone()
                        && sticker.piece(&self.puzzle.shape) == hovered_piece
                    {
                        ret.update(&self.hovered_format);
                    }
                    for clicked_piece in &self.clicked_pieces {
                        if sticker.piece(&self.puzzle.shape) == *clicked_piece {
                            ret.update(&self.clicked_format);
                        }
                    }
                    if let Some(gripped_side) = gripped_side
                        && sticker.side(&self.puzzle.shape) == gripped_side
                    {
                        ret.update(&self.gripped_format);
                    }
                    if let Some(filter_stage) = self.filter_stage {
                        for filter in &self.filter_sequence.0[filter_stage].0 {
                            if filter
                                .matches(&self.puzzle.shape, &sticker.piece(&self.puzzle.shape))
                            {
                                ret.update(&filter.format);
                            }
                        }
                        ret.build_or(&self.default_filter_format)
                    } else {
                        ret.build_or(&self.default_no_filter_format)
                    }
                };

                let painter = ui.painter();
                // TODO: pixel alignment
                let draw_position = |pos: &Position, color: Color32, format: &StickerFormat| {
                    let rect = egui::Rect::from_center_size(
                        screen_of_pos(pos),
                        egui::Vec2::new(1.0, 1.0) * scale * format.sticker_scale,
                    );
                    // TODO: is unmultiplied correct
                    painter.rect_filled(
                        rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(
                            color.r(),
                            color.g(),
                            color.b(),
                            (format.sticker_opacity * 255.0) as u8,
                        ),
                    );
                    painter.rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(format.outline_width * scale, format.outline_color),
                        egui::StrokeKind::Inside,
                    );
                };

                for pos in Position::all(&self.puzzle.shape) {
                    draw_position(&pos, self.internal_color, &self.internal_format);
                }
                for (pos, side) in &self.puzzle.stickers {
                    draw_position(&pos.0, side.color(), &format_sticker(pos));
                }

                // TODO: fancy text sizing
                let render_axis_keys = match self.turn_builder {
                    TurnBuilder::Side { side, .. } => side.is_some(),
                    TurnBuilder::Puzzle { .. } => true,
                };
                for (side, pos) in &self.side_positions {
                    if render_axis_keys && !side.is_positive() {
                        continue;
                    }
                    painter.text(
                        screen_of_pos(&pos.0),
                        egui::Align2::CENTER_CENTER,
                        if render_axis_keys {
                            side.into_axis().axis_key().to_string()
                        } else {
                            side.side_key().to_string()
                        },
                        egui::TextStyle::Monospace.resolve(&ctx.style()),
                        Color32::LIGHT_GRAY,
                    );
                }

                // painter.text(
                //     egui::Pos2::new(10.0, ui.available_height() - 10.0),
                //     egui::Align2::LEFT_BOTTOM,
                //     format!("{:?}", self.turn_builder),
                //     egui::TextStyle::Monospace.resolve(&ctx.style()),
                //     Color32::LIGHT_GRAY,
                // );
            });
    }
}

fn main() -> eframe::Result {
    // unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    // env_logger::init();

    // let mut app = App::new(&[3, 3, 4, 5, 6, 7, 8].map(Cut));
    // app.puzzle.scramble(&mut rand::rng());
    // // app.render_png("render.png");
    // std::hint::black_box(app);
    // panic!();

    // let app = App::new(&[2, 3, 4].map(Cut));
    let app = App::new(&[3, 3, 4].map(Cut));
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "rectangle",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inverse() {
        assert_eq!(
            Turn::Side(SideTurn {
                layers: LayerMask::new(),
                side: Side::new(0),
                from: Axis::new(1),
                to: Axis::new(2),
            })
            .inverse(),
            Turn::Side(SideTurn {
                layers: LayerMask::new(),
                side: Side::new(0),
                from: Axis::new(2),
                to: Axis::new(1)
            })
        );
        assert_eq!(
            Turn::Puzzle(PuzzleTurn {
                from: Axis::new(0),
                to: Axis::new(1),
            })
            .inverse(),
            Turn::Puzzle(PuzzleTurn {
                from: Axis::new(1),
                to: Axis::new(0),
            })
        );
    }

    #[test]
    fn test_is_solved() {
        for dim in 1..=4 {
            for shape in (1..=dim)
                .map(|_| (1..=4).map(Cut))
                .multi_cartesian_product()
            {
                assert!(Puzzle::new(&shape).is_solved());
            }
        }
    }

    #[test]
    fn test_turn_side_333_single() {
        let mut puzzle = Puzzle::new(&[3, 3, 3].map(Cut));
        assert!(puzzle.is_solved());
        let turn = Turn::Side(SideTurn {
            layers: LayerMask::new(),
            side: Side::new(0),
            from: Axis::new(1),
            to: Axis::new(2),
        });
        puzzle.turn(&turn).unwrap();
        assert!(!puzzle.is_solved());
        puzzle.turn(&turn.inverse()).unwrap();
        assert!(puzzle.is_solved());
    }

    #[test]
    fn test_turn_side_333_single_negative() {
        let mut puzzle = Puzzle::new(&[3, 3, 3].map(Cut));
        assert!(puzzle.is_solved());
        let turn = Turn::Side(SideTurn {
            layers: LayerMask::new(),
            side: Side::new(!0),
            from: Axis::new(1),
            to: Axis::new(2),
        });
        puzzle.turn(&turn).unwrap();
        assert!(!puzzle.is_solved());
        puzzle.turn(&turn.inverse()).unwrap();
        assert!(puzzle.is_solved());
    }

    #[test]
    fn test_turn_side_333() {
        let puzzle = Puzzle::new(&[3, 3, 3].map(Cut));
        let mut new_puzzle = puzzle.clone();
        for side in -3..3 {
            for from in 0..3 {
                if side == from || !side == from {
                    continue;
                }
                for to in 0..3 {
                    if side == to || !side == to || from == to {
                        continue;
                    }
                    // TODO: test layer mask
                    let turn = Turn::Side(SideTurn {
                        layers: LayerMask::new(),
                        side: Side::new(side),
                        from: Axis::new(from),
                        to: Axis::new(to),
                    });
                    new_puzzle.turn(&turn).unwrap();
                    assert!(!new_puzzle.is_solved());
                    new_puzzle.turn(&turn.inverse()).unwrap();
                    assert!(new_puzzle.is_solved());
                    assert_eq!(new_puzzle, puzzle);
                }
            }
        }
    }

    #[test]
    fn test_turn_puzzle_333() {
        let puzzle = Puzzle::new(&[3, 3, 3].map(Cut));
        let mut new_puzzle = puzzle.clone();
        for from in 0..3 {
            for to in 0..3 {
                if from == to {
                    continue;
                }
                let turn = Turn::Puzzle(PuzzleTurn {
                    from: Axis::new(from),
                    to: Axis::new(to),
                });
                new_puzzle.turn(&turn).unwrap();
                assert!(new_puzzle.is_solved());
                new_puzzle.turn(&turn.inverse()).unwrap();
                assert!(new_puzzle.is_solved());
                assert_eq!(new_puzzle, puzzle);
            }
        }
    }

    #[test]
    fn test_turn_side() {
        for dim in 1..=4 {
            for shape in (1..=dim)
                .map(|_| (2..=4).map(Cut))
                .multi_cartesian_product()
            {
                let puzzle = Puzzle::new(&shape);
                let mut new_puzzle = puzzle.clone();
                for side in -dim..dim {
                    for from in 0..dim {
                        if side == from || !side == from {
                            continue;
                        }
                        for to in 0..dim {
                            if side == to || !side == to || from == to {
                                continue;
                            }
                            // TODO: test layer mask
                            let turn = Turn::Side(SideTurn {
                                layers: LayerMask::new(),
                                side: Side::new(side),
                                from: Axis::new(from),
                                to: Axis::new(to),
                            });
                            new_puzzle.turn(&turn).unwrap();
                            assert!(!new_puzzle.is_solved());
                            new_puzzle.turn(&turn.inverse()).unwrap();
                            assert!(new_puzzle.is_solved());
                            assert_eq!(new_puzzle, puzzle);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_turn_puzzle() {
        for dim in 1..=4 {
            for shape in (1..=dim)
                .map(|_| (2..=4).map(Cut))
                .multi_cartesian_product()
            {
                println!("shape: {shape:?}");
                let puzzle = Puzzle::new(&shape);
                let mut new_puzzle = puzzle.clone();
                for from in 0..dim {
                    for to in 0..dim {
                        if from == to {
                            continue;
                        }
                        let turn = Turn::Puzzle(PuzzleTurn {
                            from: Axis::new(from),
                            to: Axis::new(to),
                        });
                        println!("from: {from:?}, to: {to:?}");
                        println!("here 1");
                        new_puzzle.turn(&turn).unwrap();
                        println!("here 2");
                        assert!(new_puzzle.is_solved());
                        println!("here 3");
                        new_puzzle.turn(&turn.inverse()).unwrap();
                        println!("here 4");
                        assert!(new_puzzle.is_solved());
                        println!("here 5");
                        assert_eq!(new_puzzle, puzzle);
                    }
                }
            }
        }
    }

    #[test]
    fn test_layout_mapping_inverse() {
        for dim in 1..=4 {
            for shape in (1..=dim)
                .map(|_| (2..=4).map(Cut))
                .multi_cartesian_product()
            {
                let layout = Layout2d::new(&shape);
                assert_eq!(layout.mapping.len(), layout.inverse.len());
                for (pos, xy) in &layout.mapping {
                    assert_eq!(*pos, layout.inverse[xy]);
                }
                for (xy, pos) in &layout.inverse {
                    assert_eq!(*xy, layout.mapping[pos]);
                }
            }
        }
    }

    #[test]
    fn test_sticker_side() {
        let shape = &[3, 3, 3].map(Cut);
        let sticker = Sticker::new(shape, vec![Coord(3), Coord(0), Coord(0)]);
        assert_eq!(sticker.side(shape), Side::new(0));
        let sticker = Sticker::new(shape, vec![Coord(-3), Coord(0), Coord(0)]);
        assert_eq!(sticker.side(shape), Side::new(!0));
    }

    #[test]
    fn test_piece_side() {
        let shape = &[3, 3, 3].map(Cut);
        let piece = Piece::new(shape, vec![Coord(2), Coord(0), Coord(0)]);
        assert_eq!(piece.sides(shape).collect::<Vec<_>>(), vec![Side::new(0)]);
        let piece = Piece::new(shape, vec![Coord(-2), Coord(0), Coord(0)]);
        assert_eq!(piece.sides(shape).collect::<Vec<_>>(), vec![Side::new(!0)]);
        let piece = Piece::new(shape, vec![Coord(2), Coord(2), Coord(0)]);
        assert_eq!(
            piece.sides(shape).collect::<Vec<_>>(),
            vec![Side::new(0), Side::new(1)]
        );
        let piece = Piece::new(shape, vec![Coord(-2), Coord(2), Coord(0)]);
        assert_eq!(
            piece.sides(shape).collect::<Vec<_>>(),
            vec![Side::new(!0), Side::new(1)]
        );
    }
}
