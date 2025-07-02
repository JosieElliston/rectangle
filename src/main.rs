use egui::ahash::{HashMap, HashMapExt, HashSet};
use itertools::Itertools;
use std::iter::once;

/// sides related by ! are opposite
/// rather than by -, so that we can 0 index
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
                stickers: HashMap::from_iter(
                    [(vec![Coord(-n)], Side(!0)), (vec![Coord(n)], Side(0))].into_iter(),
                ),
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
                points: HashMap::from_iter([((0, 0), vec![])].into_iter()),
                keybind_hints: if n > 2 {
                    HashMap::from_iter([((0, 0), None)].into_iter())
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
#[derive(Clone, Debug)]
struct Layout2d {
    x_lo: i16,
    x_hi: i16,
    y_lo: i16,
    y_hi: i16,
    mapping: HashMap<Vec<Coord>, (i16, i16)>,
}
impl Layout2d {
    fn new(n: i16, d: u16) -> Self {
        if d == 0 {
            panic!("idk");
        }
        if d == 1 {
            return Layout2d {
                x_lo: 0,
                x_hi: n + 1,
                y_lo: 0,
                y_hi: 0,
                mapping: HashMap::from_iter(
                    Cut(n)
                        .coords()
                        .enumerate()
                        .map(|(i, c)| (vec![c], (i as i16, 0_i16))),
                ),
            };
        }
        let horizontal = d % 2 == 1;
        let lower = Self::new(n, d - 1);
        let mut ret = Layout2d {
            x_lo: 0,
            x_hi: 0,
            y_lo: 0,
            y_hi: 0,
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
            if horizontal {
                lower.right(lower.width() * i as i16);
                assert!(ret.x_hi <= lower.x_lo);
            } else {
                lower.down(lower.height() * i as i16);
                assert!(ret.y_hi <= lower.y_lo);
            }
            ret.union(lower);
        }
        ret
    }

    fn union(&mut self, other: Self) {
        assert!(self.x_lo <= self.x_hi);
        assert!(self.y_lo <= self.y_hi);
        assert!(other.x_lo <= other.x_hi);
        assert!(other.y_lo <= other.y_hi);
        self.x_lo = self.x_lo.min(other.x_lo);
        self.x_hi = self.x_hi.max(other.x_hi);
        self.y_lo = self.y_lo.min(other.y_lo);
        self.y_hi = self.y_hi.max(other.y_hi);
        let self_len = self.mapping.len();
        let other_len = other.mapping.len();
        self.mapping.extend(other.mapping);
        assert_eq!(self.mapping.len(), self_len + other_len);
    }

    fn right(&mut self, shift: i16) {
        self.mapping.values_mut().for_each(|(x, _y)| {
            *x += shift;
        });
        self.x_lo += shift;
        self.x_hi += shift;
    }

    fn down(&mut self, shift: i16) {
        self.mapping.values_mut().for_each(|(_x, y)| {
            *y += shift;
        });
        self.y_lo += shift;
        self.y_hi += shift;
    }

    // fn new(n: i16, d: u16) -> Self {
    //     // let layout = LayoutOld::make_layout(n, d, false, false);
    //     // // let sticker_size = 1.0 / layout.width.max(layout.height) as f32;
    //     // let mut mapping = HashMap::new();
    //     // for ((x, y), pos) in layout.points {
    //     //     mapping.insert(Pos(pos.into_iter().map(Coord).collect()), (x as _, (2 * y) as _));
    //     // }
    //     // Layout2d {
    //     //     width: layout.width,
    //     //     height: layout.height,
    //     //     mapping,
    //     // }
    //     let mut x_lo = i16::MAX;
    //     let mut x_hi = i16::MIN;
    //     let mut y_lo = i16::MAX;
    //     let mut y_hi = i16::MIN;
    //     let mut mapping = HashMap::new();
    //     for pos in (0..d)
    //         .map(|_| {
    //             once(-n)
    //                 .chain((-n + 1..n).step_by(2))
    //                 .chain(once(n))
    //                 .map(Coord)
    //         })
    //         .multi_cartesian_product()
    //         // discard invalid positions
    //         // ie ones that are on multiple sides
    //         .filter(|pos| pos.iter().filter(|coord| coord.0.abs() == n).count() <= 1)
    //     {
    //         let (x, y) = Self::get(n, &pos);
    //         mapping.insert(pos, (x, y));
    //         x_lo = x_lo.min(x);
    //         x_hi = x_hi.max(x);
    //         y_lo = y_lo.min(y);
    //         y_hi = y_hi.max(y);
    //     }
    //     assert!(
    //         !mapping.is_empty(),
    //         "this might be ok, but the min and max will be wrong"
    //     );
    //     Layout2d {
    //         x_lo,
    //         x_hi,
    //         y_lo,
    //         y_hi,
    //         mapping,
    //     }
    // }

    // fn get(n: i16, pos: &[Coord]) -> (i16, i16) {
    //     if pos.len() == 0 {
    //         panic!("idk");
    //     }
    //     if pos.len() == 1 {
    //         return (
    //             once(-n)
    //                 .chain((-n + 1..n).step_by(2))
    //                 .chain(once(n))
    //                 .map(Coord)
    //                 .position(|c| c == pos[0])
    //                 .unwrap() as _,
    //             0,
    //         );
    //     }
    //     let horizontal = pos.len() % 2 == 1;
    //     let (lower_x, lower_y) = Self::get(n, &pos[..(pos.len() - 1)]);
    //     let pos_last = *pos.last().unwrap();
    //     if pos_last.0.abs() == n {
    //         // cap
    //         todo!()
    //     } else {
    //         // asdf
    //         if horizontal {
    //             (lower_x, lower_y)
    //         } else {
    //             (lower_x, lower_y)
    //         }
    //     }
    // }

    fn width(&self) -> i16 {
        self.x_hi - self.x_lo + 1
    }
    fn height(&self) -> i16 {
        self.y_hi - self.y_lo + 1
    }
}

struct App {
    puzzle: Puzzle,
    layout: Layout2d,
}
impl App {
    fn new(n: i16, d: u16) -> Self {
        let puzzle = Puzzle::new(n, d);
        let layout = Layout2d::new(n, d);
        println!(
            "x_lo: {}, x_hi: {}, y_lo: {}, y_hi: {}",
            layout.x_lo, layout.x_hi, layout.y_lo, layout.y_hi
        );
        for (pos, xy) in &layout.mapping {
            println!(
                "{:?} -> {:?}",
                pos.iter().map(|c| c.0).collect::<Vec<_>>(),
                xy,
            );
        }
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
                let scale = f32::min(
                    rect.width() / self.layout.width() as f32,
                    rect.height() / self.layout.height() as f32,
                );
                let draw_sticker = |pos: &[Coord], color: egui::Color32| {
                    // let (x, y) = self.layout.mapping.get(pos).unwrap();
                    // let rect = Rect::from_min_size(
                    //     pos.to_vec2() * square_size,
                    //     Vec2::splat(square_size),
                    // );
                    // painter.rect_filled(rect, 0.0, Color32::from_black());
                    let (x, y) = self.layout.mapping[pos];
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::Pos2::new(
                                (x - self.layout.x_lo) as f32 ,
                                (y - self.layout.y_lo) as f32 ,
                            ) * scale,
                            scale * egui::Vec2::new(1.0, 1.0),
                        ),
                        0.0,
                        color,
                    );
                };
                for pos in Cut::positions(self.puzzle.n, self.puzzle.d) {
                    draw_sticker(&pos, egui::Color32::GRAY);
                }
                for (pos, side) in &self.puzzle.stickers {
                    draw_sticker(pos, side.color());
                }
            });
    }
}

fn main() -> eframe::Result {
    // unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    // env_logger::init();

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "rectangle",
        native_options,
        // Box::new(|cc| Ok(Box::new(App::new(cc, 6, 100)))),
        Box::new(|cc| Ok(Box::new(App::new(3, 5)))),
    )
}
