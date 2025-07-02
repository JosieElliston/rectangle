use itertools::Itertools;
use std::{collections::HashMap, iter::once};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Side(i16);
impl Side {
    fn letter(self) -> char {
        const POS_NAMES: &[char] = &['R', 'U', 'F', 'O', 'A', 'Γ', 'Θ', 'Ξ', 'Σ', 'Ψ'];
        const NEG_NAMES: &[char] = &['L', 'D', 'B', 'I', 'P', 'Δ', 'Λ', 'Π', 'Φ', 'Ω'];
        if self.0 >= 0 {
            POS_NAMES[self.0 as usize]
        } else {
            NEG_NAMES[(!self.0) as usize]
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Coord(i16);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Pos(Vec<Coord>);

// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
// struct Shape(Vec<>);

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
    // map from coordinate vector (only contains -n+1, n-1 every other, and ±n)
    // to side (sides related by ! are opposite)
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
                stickers: HashMap::from([
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
                    (pos.clone().into_iter().map(Coord).collect()),
                    if side >= 0 { Side(f) } else { Side(!f) },
                );
                pos.rotate_right(1);
            }
        }
        Puzzle { n, d, stickers }
    }
}

const GAPS: &[i16] = &[0, 1, 0, 2, 1, 10, 4, 40, 18, 160, 72];
const GAPS_COMPACT: &[i16] = &[0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0];

#[derive(Debug, Clone)]
pub struct LayoutOld {
    pub width: u16,
    pub height: u16,
    pub points: HashMap<(i16, i16), Vec<i16>>,
    pub keybind_hints: HashMap<(i16, i16), Option<i16>>, // None: core, Some(i): side i
}

impl LayoutOld {
    fn new() -> Self {
        LayoutOld {
            width: 0,
            height: 0,
            points: HashMap::new(),
            keybind_hints: HashMap::new(),
        }
    }

    fn squish_right(&mut self) -> &mut Self {
        self.width = (self.points.keys().map(|(x, _y)| x).max().unwrap_or(&-1) + 1) as u16;
        self
    }

    fn squish_bottom(&mut self) -> &mut Self {
        self.height = (self.points.keys().map(|(_x, y)| y).max().unwrap_or(&-1) + 1) as u16;
        self
    }

    pub fn move_right(self, shift: i16) -> Self {
        let mut out = Self::new();
        for ((x, y), val) in &self.points {
            out.points.insert((x + shift, *y), val.to_vec());
        }
        for ((x, y), val) in &self.keybind_hints {
            out.keybind_hints.insert((x + shift, *y), *val);
        }
        out.width = (self.width as i16 + shift) as u16;
        out.height = self.height;
        out
    }

    fn move_down(self, shift: i16) -> Self {
        let mut out = Self::new();
        for ((x, y), val) in &self.points {
            out.points.insert((*x, y + shift), val.to_vec());
        }
        for ((x, y), val) in &self.keybind_hints {
            out.keybind_hints.insert((*x, y + shift), *val);
        }
        out.width = self.width;
        out.height = (self.height as i16 + shift) as u16;
        out
    }

    fn squish_left(self) -> Self {
        let shift = -self.points.keys().map(|(x, _y)| x).min().unwrap_or(&0);
        self.move_right(shift)
    }

    fn squish_top(self) -> Self {
        let shift = -self.points.keys().map(|(_x, y)| y).min().unwrap_or(&0);
        self.move_down(shift)
    }

    fn squish_horiz(self) -> Self {
        let mut out = self.squish_left();
        out.squish_right();
        out
    }

    fn squish_vert(self) -> Self {
        let mut out = self.squish_top();
        out.squish_bottom();
        out
    }

    #[allow(dead_code)]
    fn squish_all(self) -> Self {
        self.squish_horiz().squish_vert()
    }

    fn union(&mut self, other: Self) -> &mut Self {
        self.points.extend(other.points);
        self.keybind_hints.extend(other.keybind_hints);
        self.width = self.width.max(other.width);
        self.height = self.height.max(other.height);
        self
    }

    fn join_horiz(&mut self, other: Self, gap: i16) -> &mut Self {
        self.union(other.move_right(self.width as i16 + gap))
    }

    fn join_vert(&mut self, other: Self, gap: i16) -> &mut Self {
        self.union(other.move_down(self.height as i16 + gap))
    }

    fn concat_horiz(mut layouts: Vec<Self>, gap: i16) -> Self {
        let layouts_rest = layouts.split_off(1);
        let mut out = layouts
            .into_iter()
            .next()
            .expect("should have at least one element");
        for layout in layouts_rest.into_iter() {
            out.join_horiz(layout, gap);
        }
        out
    }

    fn concat_vert(mut layouts: Vec<Self>, gap: i16) -> Self {
        let layouts_rest = layouts.split_off(1);
        let mut out = layouts
            .into_iter()
            .next()
            .expect("should have at least one element");
        for layout in layouts_rest.into_iter() {
            out.join_vert(layout, gap);
        }
        out
    }

    fn clean(mut self, n: i16) -> Self {
        self.points
            .retain(|_key, val| val.iter().filter(|x| x.abs() == n).count() <= 1);
        self
    }

    fn push_all(self, x: i16) -> Self {
        let mut lower = self.clone();
        for (_xy, ref mut pos) in lower.points.iter_mut() {
            pos.push(x);
        }
        lower
    }

    pub fn make_layout(n: i16, d: u16, compact: bool, vertical: bool) -> LayoutOld {
        let gaps = if compact { GAPS_COMPACT } else { GAPS };

        if d == 0 {
            LayoutOld {
                width: 1,
                height: 1,
                points: HashMap::from([((0, 0), vec![])]),
                keybind_hints: if n > 2 {
                    HashMap::from([((0, 0), None)])
                } else {
                    HashMap::new()
                },
            }
        } else {
            let make_horizontal = d % 2 == 1 && !vertical;

            let lower = Self::make_layout(n, ((d as i16) - 1) as u16, compact, false);
            let mut row = vec![];

            for i in once(-n).chain((-n + 1..n).step_by(2)).chain(once(n)) {
                let mut lower = lower.clone().push_all(i).clean(n);
                if i.abs() == n {
                    if make_horizontal {
                        lower = lower.squish_horiz();
                    } else {
                        lower = lower.squish_vert();
                    }
                }

                lower.keybind_hints.retain(|_pos, side| {
                    let keep;
                    if i == -n + 1 {
                        keep = side.is_none();
                        *side = Some(!((d - 1) as i16));
                    } else if i == n - 1 {
                        keep = side.is_none();
                        *side = Some((d - 1) as i16);
                    } else {
                        keep = i == 0 || i == 1
                    };
                    keep
                });

                row.push(lower);
            }
            if make_horizontal {
                Self::concat_horiz(row, gaps[d as usize])
            } else {
                row.reverse();
                Self::concat_vert(row, gaps[d as usize])
            }
        }
    }
}

// TODO: this should be owned by Shape
fn all_positions(n: i16, d: u16) -> impl Iterator<Item = Vec<Coord>> {
    (0..d)
        .map(|_| {
            once(-n)
                .chain((-n + 1..n).step_by(2))
                .chain(once(n))
                .map(Coord)
        })
        .multi_cartesian_product()
        // discard invalid positions
        // ie ones that are on multiple sides
        .filter(move |pos| pos.iter().filter(|coord| coord.0.abs() == n).count() <= 1)
}

// /// mapping from Pos to [0.0, 1.0]^2
// /// +x is right, +y is up
// TODO: fix y up
// struct Layout2d {
//     sticker_size: f32,
//     // width is guaranteed to be 1.0
//     height: f32,
//     mapping: HashMap<Pos, (f32, f32)>,
// }
// impl Layout2d {
//     fn new(n: i16, d: u16) -> Self {
//         let layout = LayoutOld::make_layout(n, d, false, false);
//         let sticker_size = 1.0 / layout.width.max(layout.height) as f32;
//         let mut mapping = HashMap::new();
//         for ((x, y), pos) in layout.points {
//             mapping.insert(
//                 Pos(pos.into_iter().map(Coord).collect()),
//                 (x as f32 * sticker_size, y as f32 * 2.0 * sticker_size),
//             );
//         }
//         Layout2d {
//             sticker_size,
//             height: mapping
//                 .values()
//                 .map(|(_x, y)| *y)
//                 .reduce(f32::max)
//                 .unwrap_or(0.0),
//             mapping,
//         }
//     }
// }
// struct Layout2d {
//     width: u16,
//     height: u16,
//     mapping: HashMap<Pos, (u16, u16)>,
// }
// impl Layout2d {
//     fn new(n: i16, d: u16) -> Self {
//         let layout = LayoutOld::make_layout(n, d, false, false);
//         // let sticker_size = 1.0 / layout.width.max(layout.height) as f32;
//         let mut mapping = HashMap::new();
//         for ((x, y), pos) in layout.points {
//             mapping.insert(Pos(pos.into_iter().map(Coord).collect()), (x as _, (2 * y) as _));
//         }
//         Layout2d {
//             width: layout.width,
//             height: layout.height,
//             mapping,
//         }
//     }
// }
struct Layout2d {
    width_lo: i16,
    width_hi: i16,
    height_lo: i16,
    height_hi: i16,
    mapping: HashMap<Vec<Coord>, (i16, i16)>,
}
impl Layout2d {
    fn new(n: i16, d: u16) -> Self {
        // let layout = LayoutOld::make_layout(n, d, false, false);
        // // let sticker_size = 1.0 / layout.width.max(layout.height) as f32;
        // let mut mapping = HashMap::new();
        // for ((x, y), pos) in layout.points {
        //     mapping.insert(Pos(pos.into_iter().map(Coord).collect()), (x as _, (2 * y) as _));
        // }
        // Layout2d {
        //     width: layout.width,
        //     height: layout.height,
        //     mapping,
        // }
        let mut mapping = HashMap::new();
        let mut width_lo = i16::MAX;
        let mut width_hi = i16::MIN;
        let mut height_lo = i16::MAX;
        let mut height_hi = i16::MIN;
        for pos in (0..d)
            .map(|_| {
                once(-n)
                    .chain((-n + 1..n).step_by(2))
                    .chain(once(n))
                    .map(Coord)
            })
            .multi_cartesian_product()
            // discard invalid positions
            // ie ones that are on multiple sides
            .filter(|pos| pos.iter().filter(|coord| coord.0.abs() == n).count() <= 1)
        {
            let (x, y) = Self::get(n, &pos);
            mapping.insert(pos, (x, y));
            width_lo = width_lo.min(x);
            width_hi = width_hi.max(x);
            height_lo = height_lo.min(y);
            height_hi = height_hi.max(y);
        }
        Layout2d {
            width_lo,
            width_hi,
            height_lo,
            height_hi,
            mapping,
        }
    }

    fn get(n: i16, pos: &[Coord]) -> (i16, i16) {
        if pos.len() == 0 {
            panic!("idk");
        }
        if pos.len() == 1 {
            return (
                once(-n)
                    .chain((-n + 1..n).step_by(2))
                    .chain(once(n))
                    .map(Coord)
                    .position(|c| c == pos[0])
                    .unwrap() as _,
                0,
            );
        }
        let horizontal = pos.len() % 2 == 1;
        let (lower_x, lower_y) = Self::get(n, &pos[..(pos.len() - 1)]);
        let pos_last = *pos.last().unwrap();
        if pos_last.0.abs() == n {
            // cap
            todo!()
        } else {
            // asdf
            if horizontal {
                (lower_x, lower_y)
            } else {
                (lower_x, lower_y)
            }
        }
    }

    fn width(&self) -> u16 {
        (self.width_hi - self.width_lo + 1) as u16
    }
    fn height(&self) -> u16 {
        (self.height_hi - self.height_lo + 1) as u16
    }

    // fn map_egui(&self, rect: egui::Rect, pos: &[Coord]) -> egui::Pos2 {
    //     let scale = f32::max(
    //         rect.width() / self.width() as f32,
    //         rect.height() / self.height() as f32,
    //     );
    //     let (x, y) = self.mapping[pos];
    //     rect.left_top()
    //         + egui::Vec2::new(
    //             (x - self.width_lo) as f32 / self.width() as f32,
    //             (y - self.height_lo) as f32 / self.height() as f32,
    //         ) * scale
    // }
}

struct App {
    puzzle: Puzzle,
    layout: Layout2d,
}
impl App {
    fn new(n: i16, d: u16) -> Self {
        let puzzle = Puzzle::new(n, d);
        let layout = Layout2d::new(n, d);
        App { puzzle, layout }
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
                let scale = f32::max(
                    rect.width() / self.layout.width() as f32,
                    rect.height() / self.layout.height() as f32,
                );
                for (pos, side) in &self.puzzle.stickers {
                    // let (x, y) = self.layout.mapping.get(pos).unwrap();
                    // let rect = Rect::from_min_size(
                    //     pos.to_vec2() * square_size,
                    //     Vec2::splat(square_size),
                    // );
                    // painter.rect_filled(rect, 0.0, Color32::from_black());
                    let (x, y) = self.layout.mapping[pos];
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            rect.left_top()
                                + egui::Vec2::new(
                                    (x - self.layout.width_lo) as f32 / self.layout.width() as f32,
                                    (y - self.layout.height_lo) as f32
                                        / self.layout.height() as f32,
                                ) * scale,
                            scale * egui::Vec2::new(1.0, 1.0),
                        ),
                        0.0,
                        side.color(),
                    );
                }
            });
    }
}

fn main() -> eframe::Result {
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "particle life",
        native_options,
        // Box::new(|cc| Ok(Box::new(App::new(cc, 6, 100)))),
        Box::new(|cc| Ok(Box::new(App::new(3, 3)))),
    )
}
