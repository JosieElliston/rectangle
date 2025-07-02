use egui::ahash::{HashMap, HashMapExt};
use itertools::Itertools;
use std::iter::once;

/// sides related by ! are opposite
/// rather than by -, so that we can 0 index
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Side(i16);
impl Side {
    fn name(self) -> char {
        const POS_NAMES: &[char] = &['R', 'U', 'F', 'O', 'A', 'Γ', 'Θ', 'Ξ', 'Σ', 'Ψ'];
        const NEG_NAMES: &[char] = &['L', 'D', 'B', 'I', 'P', 'Δ', 'Λ', 'Π', 'Φ', 'Ω'];
        if self.0 >= 0 {
            POS_NAMES[self.0 as usize]
        } else {
            NEG_NAMES[(!self.0) as usize]
        }
    }

    const POS_KEYS: &[char] = &['f', 'e', 'r', 't', 'v', 'y', 'n', 'q', ',', '/'];
    const NEG_KEYS: &[char] = &['s', 'd', 'w', 'g', 'c', 'h', 'b', 'a', 'm', '.'];
    fn side_key(self) -> char {
        if self.0 >= 0 {
            Self::POS_KEYS[self.0 as usize]
        } else {
            Self::NEG_KEYS[(!self.0) as usize]
        }
    }
    fn try_from_side_key(d: u16, key: egui::Key) -> Option<Self> {
        let key = key.symbol_or_name();
        if key.len() != 1 {
            return None;
        }
        let key = key.to_lowercase().chars().next().unwrap();
        Self::POS_KEYS[..d as usize]
            .iter()
            .position(|&k| k == key)
            .map(|i| Self(i as i16))
            .or_else(|| {
                Self::NEG_KEYS[..d as usize]
                    .iter()
                    .position(|&k| k == key)
                    .map(|i| Self(!(i as i16)))
            })
    }

    const AXIS_KEYS: &[char] = &['k', 'j', 'l', 'i', 'u', 'o', 'p', ';', '[', '\''];
    fn axis_key(self) -> char {
        assert!(self.0 >= 0);
        Self::AXIS_KEYS[self.0 as usize]
    }
    fn try_from_axis_key(d: u16, key: egui::Key) -> Option<Self> {
        let key = key.symbol_or_name();
        if key.len() != 1 {
            return None;
        }
        let key = key.to_lowercase().chars().next().unwrap();
        Self::AXIS_KEYS[..d as usize]
            .iter()
            .position(|&k| k == key)
            .map(|i| Self(i as i16))
    }

    fn color(self) -> egui::Color32 {
        let pos_colors: &[egui::Color32] = &[
            egui::Color32::from_rgb(255, 0, 0),
            egui::Color32::from_rgb(255, 255, 255),
            egui::Color32::from_rgb(0, 255, 0),
            egui::Color32::from_rgb(255, 0, 255),
            egui::Color32::from_rgb(10, 170, 133),
            egui::Color32::from_rgb(119, 72, 17),
            egui::Color32::from_rgb(244, 159, 239),
            egui::Color32::from_rgb(178, 152, 103),
            egui::Color32::from_rgb(156, 245, 66),
            egui::Color32::from_rgb(7, 133, 23),
        ];
        let neg_colors: &[egui::Color32] = &[
            egui::Color32::from_rgb(255, 128, 0),
            egui::Color32::from_rgb(255, 255, 0),
            egui::Color32::from_rgb(0, 128, 255),
            egui::Color32::from_rgb(143, 16, 234),
            egui::Color32::from_rgb(125, 170, 10),
            egui::Color32::from_rgb(109, 69, 100),
            egui::Color32::from_rgb(212, 169, 78),
            egui::Color32::from_rgb(178, 121, 103),
            egui::Color32::from_rgb(66, 212, 245),
            egui::Color32::from_rgb(47, 47, 189),
        ];
        if self.0 >= 0 {
            pos_colors[self.0 as usize]
        } else {
            neg_colors[(!self.0) as usize]
        }
    }
}
impl std::ops::Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        Side(!self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Axis(Side); // but side must be nonnegative

/// lives in ±n and -n+1 to n-1 every other
/// eg for n=3, it would be in [-3, -2, 0, 2, 3]
/// eg for n=4, it would be in [-4, -3, -1, 1, 3, 4]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Coord(i16);
impl std::ops::Neg for Coord {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Coord(-self.0)
    }
}

// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
// struct Pos(Vec<Coord>);

/// A shape is a \[Cut], so a 2x3x4 would be a \[Cut(2), Cut(3), Cut(4)]
/// lives in [1, 2, 3, ...]
// TODO: better name
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cut(i16);
impl Cut {
    /// all possible coords along this axis
    fn coords(self) -> impl Iterator<Item = Coord> {
        once(-self.0)
            .chain((-self.0 + 1..self.0).step_by(2))
            .chain(once(self.0))
            .map(Coord)
    }

    /// gives all positions for this shape, including internal ones
    // fn positions(shape: &[Cut]) -> impl Iterator<Item = Vec<Coord>> {
    fn positions(n: i16, d: u16) -> impl Iterator<Item = Vec<Coord>> {
        (0..d)
            .map(|_| {
                once(-n)
                    .chain((-n + 1..n).step_by(2))
                    .chain(once(n))
                    .map(Coord)
            })
            .multi_cartesian_product()
            // discard ones that are on multiple sides, because that's impossible
            .filter(move |pos| pos.iter().filter(|coord| coord.0.abs() == n).count() <= 1)
    }
}

// #[derive(Clone, Debug, PartialEq, Eq)]
// struct Shape(Vec<Cut>);

#[derive(Clone, Debug)]
struct LayerMask(Vec<bool>);
impl LayerMask {
    fn new(n: i16) -> Self {
        let mut ret = vec![false; (n as usize - 1) / 2];
        if n > 1 {
            ret[0] = true;
        }
        LayerMask(ret)
    }
}

#[derive(Clone, Debug)]
enum Turn {
    Side {
        layers: LayerMask,
        side: Side,
        from: Side,
        to: Side,
    },
    Puzzle {
        from: Side,
        to: Side,
    },
}
impl Turn {
    fn inverse(&self) -> Self {
        match self {
            Turn::Side {
                layers,
                side,
                from,
                to,
            } => Turn::Side {
                layers: layers.clone(),
                side: *side,
                from: *to,
                to: *from,
            },
            Turn::Puzzle { from, to } => Turn::Puzzle {
                from: *to,
                to: *from,
            },
        }
    }
}

#[derive(Clone, Debug)]
enum TurnBuilder {
    Side {
        n: i16,
        d: u16,
        layers: LayerMask,
        side: Option<Side>,
        from: Option<Side>,
        // to: Option<Side>,
    },
    Puzzle {
        n: i16,
        d: u16,
        from: Option<Side>,
    },
}
impl TurnBuilder {
    fn new(n: i16, d: u16) -> Self {
        TurnBuilder::Side {
            n,
            d,
            layers: LayerMask::new(n),
            side: None,
            from: None,
        }
    }

    /// returns Some if the turn is complete
    fn update(&mut self, key: egui::Key) -> Option<Turn> {
        if key == egui::Key::Escape {
            *self = TurnBuilder::new(self.n(), self.d());
            return None;
        }
        if key == egui::Key::X {
            *self = TurnBuilder::Puzzle {
                n: self.n(),
                d: self.d(),
                from: None,
            };
            return None;
        }
        if let Ok(key) = key.name().parse::<usize>() {
            if key == 0 {
                return None;
            }
            if key > (self.n() as usize - 1) / 2 {
                return None;
            }
            match self {
                TurnBuilder::Side { layers, .. } => {
                    layers.0[key - 1] = !layers.0[key - 1];
                }
                TurnBuilder::Puzzle { n, d, .. } => {
                    *self = TurnBuilder::Side {
                        n: *n,
                        d: *d,
                        layers: LayerMask::new(*n),
                        side: None,
                        from: None,
                    };
                }
            }
            return None;
        }
        if let Some(s) = Side::try_from_side_key(self.d(), key) {
            match self {
                TurnBuilder::Side { side, from, .. } => {
                    *side = Some(s);
                    *from = None;
                }
                TurnBuilder::Puzzle { n, d, .. } => {
                    *self = TurnBuilder::Side {
                        n: *n,
                        d: *d,
                        layers: LayerMask::new(*n),
                        side: Some(s),
                        from: None,
                    };
                }
            }
            return None;
        }
        match self {
            TurnBuilder::Side {
                d,
                side,
                from,
                layers,
                ..
            } => {
                if let Some(s) = side {
                    if let Some(f) = from {
                        if let Some(t) = Side::try_from_axis_key(*d, key) {
                            let ret = Some(Turn::Side {
                                side: *s,
                                from: *f,
                                to: t,
                                layers: layers.clone(),
                            });
                            *from = None;
                            return ret;
                        }
                    } else {
                        *from = Side::try_from_axis_key(*d, key);
                    }
                }
            }
            TurnBuilder::Puzzle { d, from, .. } => {
                if let Some(f) = from {
                    if let Some(t) = Side::try_from_axis_key(*d, key) {
                        let ret = Some(Turn::Puzzle { from: *f, to: t });
                        *from = None;
                        return ret;
                    }
                } else {
                    *from = Side::try_from_axis_key(*d, key);
                }
            }
        }
        None
    }

    // TODO: these are used for evil
    fn n(&self) -> i16 {
        match self {
            TurnBuilder::Side { n, .. } => *n,
            TurnBuilder::Puzzle { n, .. } => *n,
        }
    }
    fn d(&self) -> u16 {
        match self {
            TurnBuilder::Side { d, .. } => *d,
            TurnBuilder::Puzzle { d, .. } => *d,
        }
    }
}

#[derive(Clone, Debug)]
enum TurnError {
    /// `from` and `to` don't define a plane of rotation
    UndefinedPlane,
    // Blocked, // for bandaging
}

#[derive(Clone, Debug)]
struct Puzzle {
    n: i16,
    d: u16,
    // #[serde(with = "serde_map")]
    stickers: HashMap<Vec<Coord>, Side>,
}
impl Puzzle {
    fn new(n: i16, d: u16) -> Self {
        if d == 1 {
            // i think multi_cartesian_product returns empty iterator for the empty product

            return Puzzle {
                n,
                d,
                stickers: HashMap::from_iter([
                    (vec![Coord(-n)], Side(!0)),
                    (vec![Coord(n)], Side(0)),
                ]),
            };
        }

        let mut stickers = HashMap::new();
        for (side, coords) in [n, -n].into_iter().cartesian_product(
            (0..d - 1)
                .map(|_| (-n + 1..n).step_by(2))
                .multi_cartesian_product(),
        ) {
            let mut pos = vec![side];
            pos.extend(&coords);
            for f in 0..(d as i16) {
                // TODO: this is bad
                stickers.insert(
                    pos.clone().into_iter().map(Coord).collect(),
                    if side >= 0 { Side(f) } else { Side(!f) },
                );
                pos.rotate_right(1);
            }
        }
        Puzzle { n, d, stickers }
    }

    fn is_solved(&self) -> bool {
        let mut pos_colors: Vec<Option<Side>> = vec![None; self.d as usize];
        let mut neg_colors: Vec<Option<Side>> = vec![None; self.d as usize];
        for (pos, color) in &self.stickers {
            assert!(pos.iter().filter(|c| c.0.abs() == self.n).count() == 1);
            let axis = pos.iter().position(|c| c.0.abs() == self.n).unwrap();
            let side = if pos[axis].0 >= 0 {
                Side(axis as i16)
            } else {
                Side(!(axis as i16))
            };
            let e = if side.0 >= 0 {
                &mut pos_colors[side.0 as usize]
            } else {
                &mut neg_colors[(!side.0) as usize]
            };
            match e {
                None => *e = Some(*color),
                Some(color) => {
                    if *color != side {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn turn(&mut self, turn: &Turn) -> Result<(), TurnError> {
        match turn {
            Turn::Side {
                layers,
                side,
                from,
                to,
            } => {
                if side == from
                    || *side == !*from
                    || side == to
                    || *side == !*to
                    || from == to
                    || *from == !*to
                {
                    return Err(TurnError::UndefinedPlane);
                }
                assert!(from.0 >= 0 && to.0 >= 0);
                // TODO: i don't think this needs to be a hashmap
                let mut new_stickers = HashMap::new();
                for pos in self.stickers.keys() {
                    // TODO: layer mask
                    // if if side.0 >= 0 {
                    //     layers.0[pos[side.0 as usize].0 as usize]
                    // } else {
                    //     layers.0[pos[(!side.0) as usize].0 as usize]
                    // } {
                    if if side.0 >= 0 {
                        ((self.n - 1)..=self.n).contains(&pos[side.0 as usize].0)
                    } else {
                        ((-self.n)..=(1 - self.n)).contains(&pos[(!side.0) as usize].0)
                    } {
                        println!("here");
                        // TODO: compute to_pos instead of from_pos???
                        let mut from_pos = pos.clone();
                        from_pos[from.0 as usize] = pos[to.0 as usize];
                        from_pos[to.0 as usize] = -pos[from.0 as usize];
                        new_stickers.insert(pos.clone(), self.stickers[&from_pos]);
                    }
                }
                self.stickers.extend(new_stickers);

                Ok(())
            }
            Turn::Puzzle { from, to } => {
                if from == to || *from == !*to {
                    return Err(TurnError::UndefinedPlane);
                }
                let mut new_stickers = HashMap::new();
                for pos in self.stickers.keys() {
                    let mut from_pos = pos.clone();
                    from_pos[from.0 as usize] = pos[to.0 as usize];
                    from_pos[to.0 as usize] = -pos[from.0 as usize];
                    new_stickers.insert(pos.clone(), self.stickers[&from_pos]);
                }
                self.stickers = new_stickers;
                Ok(())
            }
        }
    }
}

/// mapping from Pos to (x, y) coordinates
/// +x is right, +y is up
#[derive(Clone, Debug)]
struct Layout2d {
    width: usize,
    height: usize,
    mapping: HashMap<Vec<Coord>, (usize, usize)>,
}
impl Layout2d {
    fn new(n: i16, d: u16) -> Self {
        if d == 0 {
            return Self {
                width: 1,
                height: 1,
                mapping: HashMap::from_iter([(vec![], (0, 0))]),
            };
        }
        // if d == 1 {
        //     return Layout2d {
        //         x_hi: n + 1,
        //         height: 0,
        //         mapping: HashMap::from_iter(
        //             Cut(n)
        //                 .coords()
        //                 .enumerate()
        //                 .map(|(i, c)| (vec![c], (i as i16, 0_i16))),
        //         ),
        //     };
        // }
        let horizontal = d % 2 == 1;
        let lower = Self::new(n, d - 1);
        let mut ret = Self {
            width: 0,
            height: 0,
            mapping: HashMap::new(),
        };
        for (i, new_coord) in Cut(n).coords().enumerate() {
            let mut lower = lower.clone();
            lower.mapping = lower
                .mapping
                .into_iter()
                .map(|(mut pos, xy)| {
                    pos.push(new_coord);
                    (pos, xy)
                })
                .collect();

            if new_coord.0.abs() == n {
                lower
                    .mapping
                    .retain(|pos, _xy| pos.iter().filter(|coord| coord.0.abs() == n).count() <= 1);
                // TODO: possibly shrink margins
            } else {
                debug_assert!(lower.mapping.iter().all(|(pos, _xy)| {
                    pos.iter().filter(|coord| coord.0.abs() == n).count() <= 1
                }));
            }
            let shift = if horizontal {
                lower.width
            } else {
                lower.height
            };
            let shift = if d > 2 { (shift + 1) * i } else { shift * i };
            if horizontal {
                lower.right(shift);
                // assert!(ret.x_hi <= lower.x_lo);
            } else {
                lower.down(shift);
                // assert!(ret.height <= lower.y_lo);
            }
            ret.union(lower);
        }
        ret
    }

    fn union(&mut self, other: Self) {
        self.width = self.width.max(other.width);
        self.height = self.height.max(other.height);
        let self_len = self.mapping.len();
        let other_len = other.mapping.len();
        self.mapping.extend(other.mapping);
        assert_eq!(self.mapping.len(), self_len + other_len);
    }

    fn right(&mut self, shift: usize) {
        self.mapping.values_mut().for_each(|(x, _y)| {
            *x += shift;
        });
        self.width += shift;
    }

    fn down(&mut self, shift: usize) {
        self.mapping.values_mut().for_each(|(_x, y)| {
            *y += shift;
        });
        self.height += shift;
    }
}

enum Layout {
    // OneD(Layout1d),
    TwoD(Layout2d),
    // ThreeD(Layout3d),
}

struct App {
    puzzle: Puzzle,
    layout: Layout,
    /// where the labels for the sides go
    side_positions: HashMap<Side, Vec<Coord>>,
    turn_builder: TurnBuilder,
}
impl App {
    fn new(n: i16, d: u16) -> Self {
        const MAX_DIM: u16 = 10;
        const MAX_LAYERS: i16 = 19;
        assert!(d > 0, "dimension should be greater than 0");
        assert!(
            d <= MAX_DIM,
            "dimension should be less than or equal to {MAX_DIM}"
        );
        assert!(n > 0, "side should be greater than 0");
        assert!(
            n <= MAX_LAYERS,
            "side should be less than or equal to {MAX_LAYERS}"
        );
        // let start = std::time::Instant::now();
        let puzzle = Puzzle::new(n, d);
        // println!("puzzle gen in {:?}", start.elapsed());

        // let start = std::time::Instant::now();
        let layout = Layout::TwoD(Layout2d::new(n, d));
        // println!("layout gen in {:?}", start.elapsed());

        let mut side_positions = HashMap::new();
        for side in 0..d as i16 {
            {
                // positive
                let mut pos = vec![if n % 2 == 1 { Coord(0) } else { Coord(1) }; d as usize];
                pos[side as usize] = Coord(n - 1);
                side_positions.insert(Side(side), pos);
            }
            {
                // negative
                let mut pos = vec![if n % 2 == 1 { Coord(0) } else { Coord(1) }; d as usize];
                pos[side as usize] = Coord(1 - n);
                side_positions.insert(Side(!side), pos);
            }
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
            side_positions,
            turn_builder: TurnBuilder::new(n, d),
        }
    }

    fn render_png(&self, path: &str) {
        let Layout::TwoD(layout) = &self.layout else {
            panic!("render_png only works for Layout2d");
        };
        let start = std::time::Instant::now();
        let mut buf = vec![0; layout.width * layout.height * 3];

        let mut draw_sticker = |pos: &[Coord], color: egui::Color32| {
            let (x, y) = layout.mapping[pos];
            let i = ((layout.height - y - 1) * layout.width + x) * 3;
            buf[i] = color.r();
            buf[i + 1] = color.g();
            buf[i + 2] = color.b();
        };

        for pos in Cut::positions(self.puzzle.n, self.puzzle.d) {
            draw_sticker(&pos, egui::Color32::GRAY);
        }
        for (pos, side) in &self.puzzle.stickers {
            draw_sticker(pos, side.color());
        }
        println!("buf gen in {:?}", start.elapsed());

        let start = std::time::Instant::now();
        image::save_buffer(
            path,
            &buf,
            layout.width as _,
            layout.height as _,
            image::ColorType::Rgb8,
        )
        .unwrap();
        println!("image save in {:?}", start.elapsed());
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ctx.input(|i| {
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
                                }
                            }
                        }
                    }
                });

                // let dt = ctx.input(|input_state| input_state.stable_dt);
                // // println!("dt: {:?}", dt);
                match &self.layout {
                    Layout::TwoD(layout) => {
                        let painter = ui.painter();
                        let rect = ui.available_rect_before_wrap();
                        let scale = f32::min(
                            rect.width() / layout.width as f32,
                            rect.height() / layout.height as f32,
                        );
                        // TODO: pixel alignment
                        let rect_of_sticker = |pos: &[Coord]| {
                            let (x, y) = layout.mapping[pos];
                            egui::Rect::from_min_size(
                                egui::Pos2::new(x as f32, (layout.height - y - 1) as f32) * scale,
                                scale * egui::Vec2::new(1.0, 1.0),
                            )
                        };
                        let draw_sticker = |pos: &[Coord], color: egui::Color32| {
                            painter.rect_filled(rect_of_sticker(pos), 0.0, color);
                        };
                        for pos in Cut::positions(self.puzzle.n, self.puzzle.d) {
                            draw_sticker(&pos, egui::Color32::DARK_GRAY);
                        }
                        for (pos, side) in &self.puzzle.stickers {
                            draw_sticker(pos, side.color());
                        }

                        // TODO: fancy text sizing
                        let render_axis_keys = match self.turn_builder {
                            TurnBuilder::Side { side, .. } => side.is_some(),
                            TurnBuilder::Puzzle { .. } => true,
                        };
                        for (side, pos) in &self.side_positions {
                            if render_axis_keys && side.0 < 0 {
                                continue;
                            }
                            painter.text(
                                rect_of_sticker(pos).center(),
                                egui::Align2::CENTER_CENTER,
                                if render_axis_keys {
                                    side.axis_key().to_string()
                                } else {
                                    side.side_key().to_string()
                                },
                                egui::TextStyle::Monospace.resolve(&ctx.style()),
                                egui::Color32::LIGHT_GRAY,
                            );
                        }
                    }
                }

                // painter.text(
                //     egui::Pos2::new(10.0, ui.available_height() - 10.0),
                //     egui::Align2::LEFT_BOTTOM,
                //     format!("{:?}", self.turn_builder),
                //     egui::TextStyle::Monospace.resolve(&ctx.style()),
                //     egui::Color32::LIGHT_GRAY,
                // );
            });
    }
}

fn main() -> eframe::Result {
    // unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    // env_logger::init();

    // let app = App::new(3, 10);
    // app.render_png("render.png");
    // panic!();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "rectangle",
        native_options,
        Box::new(|_cc| Ok(Box::new(App::new(3, 3)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_solved() {
        let mut puzzle = Puzzle::new(3, 3);
        assert!(puzzle.is_solved());
        puzzle
            .turn(&Turn::Puzzle {
                from: Side(0),
                to: Side(1),
            })
            .unwrap();
        assert!(puzzle.is_solved());
        let turn = Turn::Side {
            layers: LayerMask(vec![true]),
            side: Side(0),
            from: Side(1),
            to: Side(2),
        };
        puzzle.turn(&turn).unwrap();
        assert!(!puzzle.is_solved());
        puzzle.turn(&turn.inverse()).unwrap();
        assert!(puzzle.is_solved());
    }
}
