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
        // TODO: make this const
        let pos_colors: &[egui::Color32] = &[
            egui::Color32::from_hex("#ff0000").unwrap(),
            egui::Color32::from_hex("#ffffff").unwrap(),
            egui::Color32::from_hex("#00ff00").unwrap(),
            egui::Color32::from_hex("#ff00ff").unwrap(),
            egui::Color32::from_hex("#0aaa85").unwrap(),
            egui::Color32::from_hex("#774811").unwrap(),
            egui::Color32::from_hex("#f49fef").unwrap(),
            egui::Color32::from_hex("#b29867").unwrap(),
            egui::Color32::from_hex("#9cf542").unwrap(),
            egui::Color32::from_hex("#078517").unwrap(),
        ];
        let neg_colors: &[egui::Color32] = &[
            egui::Color32::from_hex("#ff8000").unwrap(),
            egui::Color32::from_hex("#ffff00").unwrap(),
            egui::Color32::from_hex("#0080ff").unwrap(),
            egui::Color32::from_hex("#8f10ea").unwrap(),
            egui::Color32::from_hex("#7daa0a").unwrap(),
            egui::Color32::from_hex("#6d4564").unwrap(),
            egui::Color32::from_hex("#d4a94e").unwrap(),
            egui::Color32::from_hex("#b27967").unwrap(),
            egui::Color32::from_hex("#42d4f5").unwrap(),
            egui::Color32::from_hex("#2f2fbd").unwrap(),
        ];
        if self.0 >= 0 {
            pos_colors[self.0 as usize]
        } else {
            neg_colors[(!self.0) as usize]
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Axis(Side); // but side must be nonnegative

/// lives in ±n and -n+1 to n-1 every other
/// eg for n=3, it would be in [-3, -2, 0, 2, 3]
/// eg for n=4, it would be in [-4, -3, -1, 1, 3, 4]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Coord(i16);

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
struct SideTurn {
    layers: LayerMask,
    side: Side,
    from: Side,
    to: Side,
}

#[derive(Clone, Debug)]
struct PuzzleTurn {
    from: Side,
    to: Side,
}

#[derive(Clone, Debug)]
enum Turn {
    Side(SideTurn),
    Puzzle(PuzzleTurn),
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
        // to: Option<Side>,
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
                // to: None,
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
                        // to: None,
                    };
                }
            }
            return None;
        }
        if let Some(s) = Side::try_from_side_key(self.d(), key) {
            match self {
                TurnBuilder::Side {
                    n,
                    d,
                    layers,
                    side,
                    from,
                    // to,
                } => {
                    *side = Some(s);
                    *from = None;
                    // *to = None;
                }
                TurnBuilder::Puzzle { n, d, .. } => {
                    *self = TurnBuilder::Side {
                        n: *n,
                        d: *d,
                        layers: LayerMask::new(*n),
                        side: Some(s),
                        from: None,
                        // to: None,
                    };
                }
            }
            return None;
        }
        match self {
            TurnBuilder::Side {
                n,
                d,
                side,
                from,
                // to,
                layers,
            } => {
                if let Some(s) = side {
                    if let Some(f) = from {
                        if let Some(t) = Side::try_from_axis_key(*d, key) {
                            let ret = Some(Turn::Side(SideTurn {
                                side: *s,
                                from: *f,
                                to: t,
                                layers: layers.clone(),
                            }));
                            *from = None;
                            // *to = None;
                            return ret;
                        }
                    } else {
                        *from = Side::try_from_axis_key(*d, key);
                    }
                }
            }
            TurnBuilder::Puzzle { n, d, from } => {
                if let Some(f) = from {
                    if let Some(t) = Side::try_from_axis_key(*d, key) {
                        let ret = Some(Turn::Puzzle(PuzzleTurn { from: *f, to: t }));
                        *from = None;
                        // *to = None;
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

    fn turn(&mut self, turn: Turn) {
        println!("TODO");
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

struct App {
    puzzle: Puzzle,
    layout: Layout2d,
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
        let start = std::time::Instant::now();
        let puzzle = Puzzle::new(n, d);
        println!("puzzle gen in {:?}", start.elapsed());

        let start = std::time::Instant::now();
        let layout = Layout2d::new(n, d);
        println!("layout gen in {:?}", start.elapsed());

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
        let start = std::time::Instant::now();
        let mut buf = vec![0; self.layout.width * self.layout.height * 3];

        let mut draw_sticker = |pos: &[Coord], color: egui::Color32| {
            let (x, y) = self.layout.mapping[pos];
            let i = ((self.layout.height - y - 1) * self.layout.width + x) * 3;
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
            self.layout.width as _,
            self.layout.height as _,
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
                            physical_key,
                            pressed,
                            repeat,
                            modifiers,
                        } = event
                        {
                            if *pressed && !repeat {
                                if let Some(turn) = self.turn_builder.update(*key) {
                                    self.puzzle.turn(turn);
                                }
                            }
                        }
                    }
                });

                // let dt = ctx.input(|input_state| input_state.stable_dt);
                // // println!("dt: {:?}", dt);
                // let scale = ui.available_rect_before_wrap().size().min_elem();
                // let scale = ui.available_rect_before_wrap().width();
                let painter = ui.painter();
                // let sticker_size =
                //     ui.available_rect_before_wrap().width() / self.layout.width() as f32;
                let rect = ui.available_rect_before_wrap();
                let scale = f32::min(
                    rect.width() / self.layout.width as f32,
                    rect.height() / self.layout.height as f32,
                );
                // TODO: pixel alignment
                let rect_of_sticker = |pos: &[Coord]| {
                    let (x, y) = self.layout.mapping[pos];
                    egui::Rect::from_min_size(
                        egui::Pos2::new(x as f32, (self.layout.height - y - 1) as f32) * scale,
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

                painter.text(
                    egui::Pos2::new(10.0, ui.available_height() - 10.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!("{:?}", self.turn_builder),
                    egui::TextStyle::Monospace.resolve(&ctx.style()),
                    egui::Color32::LIGHT_GRAY,
                );
            });
    }
}

fn main() -> eframe::Result {
    // unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    // env_logger::init();

    let app = App::new(3, 10);
    app.render_png("render.png");
    panic!();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "rectangle",
        native_options,
        Box::new(|_cc| Ok(Box::new(App::new(3, 3)))),
    )
}
