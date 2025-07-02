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

    fn grip_key(self) -> char {
        todo!()
    }
    // TODO: better name
    fn grip_key_right(self) -> char {
        todo!()
    }
    // const POS_KEYS: &[char] = &['f', 'e', 'r', 't', 'v', 'y', 'n', 'q', ',', '/'];
    // const NEG_KEYS: &[char] = &['s', 'd', 'w', 'g', 'c', 'h', 'b', 'a', 'm', '.'];
    // const POS_KEYS_RIGHT: &[char] = &['l', 'i', 'j', '.', 'p', '['];
    // const NEG_KEYS_RIGHT: &[char] = &['u', ',', 'o', 'k', 'l', ';'];

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

#[derive(Clone, Debug)]
struct SideTurn {
    side: Side,
    from: Side,
    to: Side,
    layers: LayerMask,
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
            return Layout2d {
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
        let mut ret = Layout2d {
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

        let puzzle = Puzzle::new(n, d);
        let layout = Layout2d::new(n, d);
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
        }
    }

    fn render_png(&self, path: &str) {
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

        image::save_buffer(
            path,
            &buf,
            self.layout.width as _,
            self.layout.height as _,
            image::ColorType::Rgb8,
        )
        .unwrap()
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
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
                    let (x, y) = self.layout.mapping[pos];
                    painter.rect_filled(rect_of_sticker(pos), 0.0, color);
                };
                for pos in Cut::positions(self.puzzle.n, self.puzzle.d) {
                    draw_sticker(&pos, egui::Color32::DARK_GRAY);
                }
                for (pos, side) in &self.puzzle.stickers {
                    draw_sticker(pos, side.color());
                }
                for (side, pos) in &self.side_positions {
                    // TODO: fancy text sizing
                    painter.text(
                        rect_of_sticker(pos).center(),
                        egui::Align2::CENTER_CENTER,
                        side.name().to_string(),
                        egui::TextStyle::Monospace.resolve(&ctx.style()),
                        egui::Color32::LIGHT_GRAY,
                    );
                }
            });
    }
}

fn main() -> eframe::Result {
    // unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    // env_logger::init();

    let native_options = eframe::NativeOptions::default();
    // let app = App::new(3, 4);
    // app.render_png("render.png");
    // panic!();
    eframe::run_native(
        "rectangle",
        native_options,
        Box::new(|_cc| Ok(Box::new(App::new(3, 3)))),
    )
}
