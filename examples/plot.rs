use plotters::chart::{ChartBuilder, ChartContext, SeriesLabelPosition};
use plotters::coord::{cartesian::Cartesian2d, combinators::IntoLogRange, ranged1d::Ranged, Shift};
use plotters::drawing::{DrawingArea, IntoDrawingArea};
use plotters::element::{
    Circle, Drawable, DynElement, EmptyElement, IntoDynElement, PathElement, PointCollection,
};
use plotters::prelude::SVGBackend;
use plotters::series::LineSeries;
use plotters::style::colors::{BLACK, WHITE};
use plotters::style::{Color, Palette, Palette99, RGBAColor, ShapeStyle, SizeDesc};
use plotters_backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use regex::Regex;
use std::cmp;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let width = 800;
    let height = 600;
    println!("Drawing area: {}, {}", width, height);

    let line_styles: &[LineStyle] = &[
        LineStyle {
            color: Palette99::pick(0).to_rgba(),
            decorator: Decorator::Circle {
                size: 4,
                filled: true,
            },
        },
        LineStyle {
            color: Palette99::pick(1).to_rgba(),
            decorator: Decorator::Square {
                size: 4,
                filled: true,
            },
        },
        LineStyle {
            color: Palette99::pick(3).to_rgba(),
            decorator: Decorator::Triangle {
                size: 6,
                filled: true,
            },
        },
        LineStyle {
            color: Palette99::pick(4).to_rgba(),
            decorator: Decorator::Circle {
                size: 4,
                filled: false,
            },
        },
        LineStyle {
            color: Palette99::pick(5).to_rgba(),
            decorator: Decorator::Square {
                size: 4,
                filled: false,
            },
        },
        LineStyle {
            color: Palette99::pick(7).to_rgba(),
            decorator: Decorator::Triangle {
                size: 6,
                filled: false,
            },
        },
    ];

    let benches = parse(BufReader::new(File::open("benchmark.txt").unwrap()));
    let drawing_area = SVGBackend::new("plot.svg", (width, height)).into_drawing_area();
    draw_bench("Benchmarks", drawing_area, line_styles, &benches);

    let benches_native = parse(BufReader::new(File::open("benchmark-native.txt").unwrap()));
    let drawing_area = SVGBackend::new("plot-native.svg", (width, height)).into_drawing_area();
    draw_bench(
        "Benchmarks (native)",
        drawing_area,
        line_styles,
        &benches_native,
    );

    let benches_native_clmul = parse(BufReader::new(
        File::open("benchmark-native-clmul.txt").unwrap(),
    ));
    let drawing_area =
        SVGBackend::new("plot-native-clmul.svg", (width, height)).into_drawing_area();
    draw_bench(
        "Benchmarks (clmul)",
        drawing_area,
        line_styles,
        &benches_native_clmul,
    );

    let drawing_area = SVGBackend::new("plot-field-ops.svg", (width, height)).into_drawing_area();
    draw_field_ops(
        drawing_area,
        line_styles.iter(),
        &benches,
        &benches_native,
        &benches_native_clmul,
    );

    let drawing_area = SVGBackend::new("plot-compact.svg", (width, height)).into_drawing_area();
    draw_compact(
        drawing_area,
        line_styles.iter(),
        &benches,
        &benches_native,
        &benches_native_clmul,
    );

    let drawing_area = SVGBackend::new("plot-random.svg", (width, height)).into_drawing_area();
    draw_random(
        drawing_area,
        line_styles.iter(),
        &benches,
        &benches_native,
        &benches_native_clmul,
    );
}

fn draw_field_ops<'a, DB: DrawingBackend>(
    drawing_area: DrawingArea<DB, Shift>,
    mut line_styles: impl Iterator<Item = &'a LineStyle>,
    benches: &[Bench],
    benches_native: &[Bench],
    benches_native_clmul: &[Bench],
) {
    let (min, max) = chart_limits(
        benches
            .iter()
            .chain(benches_native.iter())
            .chain(benches_native_clmul.iter()),
        &["bench_mul", "bench_invert"],
    );

    let mut chart = ChartBuilder::on(&drawing_area)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .margin(10)
        .margin_right(40)
        .caption("Field operations", ("sans-serif", 30))
        .build_cartesian_2d(0..8, (min..max).log_scale())
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .label_style(("sans-serif", 14))
        .x_desc("Field")
        .y_desc("Time")
        .x_label_formatter(&|&v| field_formatter(v).to_owned())
        .y_label_formatter(&|&v| {
            if v < 1000 {
                format!("{} ns", v)
            } else if v < 1_000_000 {
                format!("{} µs", v as f64 / 1e3)
            } else if v < 1_000_000_000 {
                format!("{} ms", v as f64 / 1e6)
            } else {
                format!("{} s", v as f64 / 1e9)
            }
        })
        .draw()
        .unwrap();

    line_bench(
        &mut chart,
        benches,
        "bench_mul",
        Some("mul"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native,
        "bench_mul",
        Some("mul (native)"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native_clmul,
        "bench_mul",
        Some("mul (clmul)"),
        line_styles.next().unwrap(),
    );

    line_bench(
        &mut chart,
        benches,
        "bench_invert",
        Some("invert"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native,
        "bench_invert",
        Some("invert (native)"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native_clmul,
        "bench_invert",
        Some("invert (clmul)"),
        line_styles.next().unwrap(),
    );

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .border_style(&BLACK)
        .background_style(WHITE.filled())
        .label_font(("sans-serif", 14))
        .draw()
        .unwrap();
}

fn draw_compact<'a, DB: DrawingBackend>(
    drawing_area: DrawingArea<DB, Shift>,
    mut line_styles: impl Iterator<Item = &'a LineStyle>,
    benches: &[Bench],
    benches_native: &[Bench],
    benches_native_clmul: &[Bench],
) {
    let (min, max) = chart_limits(
        benches
            .iter()
            .chain(benches_native.iter())
            .chain(benches_native_clmul.iter()),
        &["compact::bench_split_10", "compact::bench_reconstruct_10"],
    );

    let mut chart = ChartBuilder::on(&drawing_area)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .margin(10)
        .margin_right(40)
        .caption("Shamir (compact)", ("sans-serif", 30))
        .build_cartesian_2d(0..8, (min..max).log_scale())
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .label_style(("sans-serif", 14))
        .x_desc("Field")
        .y_desc("Time")
        .x_label_formatter(&|&v| field_formatter(v).to_owned())
        .y_label_formatter(&|&v| {
            if v < 1000 {
                format!("{} ns", v)
            } else if v < 1_000_000 {
                format!("{} µs", v as f64 / 1e3)
            } else if v < 1_000_000_000 {
                format!("{} ms", v as f64 / 1e6)
            } else {
                format!("{} s", v as f64 / 1e9)
            }
        })
        .draw()
        .unwrap();

    line_bench(
        &mut chart,
        benches,
        "compact::bench_split_10",
        Some("split_10"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native,
        "compact::bench_split_10",
        Some("split_10 (native)"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native_clmul,
        "compact::bench_split_10",
        Some("split_10 (clmul)"),
        line_styles.next().unwrap(),
    );

    line_bench(
        &mut chart,
        benches,
        "compact::bench_reconstruct_10",
        Some("reconstruct_10"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native,
        "compact::bench_reconstruct_10",
        Some("reconstruct_10 (native)"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native_clmul,
        "compact::bench_reconstruct_10",
        Some("reconstruct_10 (clmul)"),
        line_styles.next().unwrap(),
    );

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .border_style(&BLACK)
        .background_style(WHITE.filled())
        .label_font(("sans-serif", 14))
        .draw()
        .unwrap();
}

fn draw_random<'a, DB: DrawingBackend>(
    drawing_area: DrawingArea<DB, Shift>,
    mut line_styles: impl Iterator<Item = &'a LineStyle>,
    benches: &[Bench],
    benches_native: &[Bench],
    benches_native_clmul: &[Bench],
) {
    let (min, max) = chart_limits(
        benches
            .iter()
            .chain(benches_native.iter())
            .chain(benches_native_clmul.iter()),
        &["random::bench_split_10", "random::bench_reconstruct_10"],
    );

    let mut chart = ChartBuilder::on(&drawing_area)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .margin(10)
        .margin_right(40)
        .caption("Shamir (random)", ("sans-serif", 30))
        .build_cartesian_2d(0..8, (min..max).log_scale())
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .label_style(("sans-serif", 14))
        .x_desc("Field")
        .y_desc("Time")
        .x_label_formatter(&|&v| field_formatter(v).to_owned())
        .y_label_formatter(&|&v| {
            if v < 1000 {
                format!("{} ns", v)
            } else if v < 1_000_000 {
                format!("{} µs", v as f64 / 1e3)
            } else if v < 1_000_000_000 {
                format!("{} ms", v as f64 / 1e6)
            } else {
                format!("{} s", v as f64 / 1e9)
            }
        })
        .draw()
        .unwrap();

    line_bench(
        &mut chart,
        benches,
        "random::bench_split_10",
        Some("split_10"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native,
        "random::bench_split_10",
        Some("split_10 (native)"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native_clmul,
        "random::bench_split_10",
        Some("split_10 (clmul)"),
        line_styles.next().unwrap(),
    );

    line_bench(
        &mut chart,
        benches,
        "random::bench_reconstruct_10",
        Some("reconstruct_10"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native,
        "random::bench_reconstruct_10",
        Some("reconstruct_10 (native)"),
        line_styles.next().unwrap(),
    );
    line_bench(
        &mut chart,
        benches_native_clmul,
        "random::bench_reconstruct_10",
        Some("reconstruct_10 (clmul)"),
        line_styles.next().unwrap(),
    );

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .border_style(&BLACK)
        .background_style(WHITE.filled())
        .label_font(("sans-serif", 14))
        .draw()
        .unwrap();
}

fn draw_bench<DB: DrawingBackend>(
    title: &str,
    drawing_area: DrawingArea<DB, Shift>,
    line_styles: &[LineStyle],
    benches: &[Bench],
) {
    let (min, max) = chart_limits(
        benches.iter(),
        &[
            "bench_mul",
            "bench_invert",
            "compact::bench_split_10",
            "compact::bench_reconstruct_10",
            "compact::bench_split_big_all",
            "compact::bench_reconstruct_big_all",
        ],
    );

    let mut chart = ChartBuilder::on(&drawing_area)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .margin(10)
        .margin_right(40)
        .caption(title, ("sans-serif", 30))
        .build_cartesian_2d(0..8, (min..max).log_scale())
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .label_style(("sans-serif", 14))
        .x_desc("Field")
        .y_desc("Time")
        .x_label_formatter(&|&v| field_formatter(v).to_owned())
        .y_label_formatter(&|&v| {
            if v < 1000 {
                format!("{} ns", v)
            } else if v < 1_000_000 {
                format!("{} µs", v as f64 / 1e3)
            } else if v < 1_000_000_000 {
                format!("{} ms", v as f64 / 1e6)
            } else {
                format!("{} s", v as f64 / 1e9)
            }
        })
        .draw()
        .unwrap();

    line_bench(
        &mut chart,
        benches,
        "bench_mul",
        Some("mul"),
        &line_styles[0],
    );
    line_bench(
        &mut chart,
        benches,
        "bench_invert",
        Some("invert"),
        &line_styles[3],
    );

    line_bench(
        &mut chart,
        benches,
        "compact::bench_split_10",
        Some("split_10"),
        &line_styles[1],
    );
    line_bench(
        &mut chart,
        benches,
        "compact::bench_reconstruct_10",
        Some("reconstruct_10"),
        &line_styles[4],
    );

    line_bench(
        &mut chart,
        benches,
        "compact::bench_split_big_all",
        Some("split_255"),
        &line_styles[2],
    );
    line_bench(
        &mut chart,
        benches,
        "compact::bench_reconstruct_big_all",
        Some("reconstruct_255"),
        &line_styles[5],
    );

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerRight)
        .border_style(&BLACK)
        .background_style(WHITE.filled())
        .label_font(("sans-serif", 14))
        .draw()
        .unwrap();
}

struct LineStyle {
    color: RGBAColor,
    decorator: Decorator,
}

enum Decorator {
    Circle { size: u32, filled: bool },
    Triangle { size: u32, filled: bool },
    Square { size: u32, filled: bool },
}

impl Decorator {
    fn decorate<DB, Coord>(&self, coord: Coord, color: RGBAColor) -> DynElement<'static, DB, Coord>
    where
        DB: DrawingBackend,
        Coord: Clone + 'static,
    {
        match self {
            Decorator::Circle { size, filled } => Circle::new(
                coord,
                *size,
                if *filled {
                    color.filled()
                } else {
                    color.into()
                },
            )
            .into_dyn(),
            Decorator::Triangle { size, filled } => Triangle::new(
                coord,
                *size,
                if *filled {
                    color.filled()
                } else {
                    color.into()
                },
            )
            .into_dyn(),
            Decorator::Square { size, filled } => Square::new(
                coord,
                *size,
                if *filled {
                    color.filled()
                } else {
                    color.into()
                },
            )
            .into_dyn(),
        }
    }
}

pub struct Square<Coord, Size: SizeDesc> {
    center: Coord,
    size: Size,
    style: ShapeStyle,
}

impl<Coord, Size: SizeDesc> Square<Coord, Size> {
    pub fn new<T: Into<ShapeStyle>>(coord: Coord, size: Size, style: T) -> Self {
        Self {
            center: coord,
            size,
            style: style.into(),
        }
    }
}

impl<'a, Coord: 'a, Size: SizeDesc> PointCollection<'a, Coord> for &'a Square<Coord, Size> {
    type Point = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        std::iter::once(&self.center)
    }
}

impl<Coord, DB: DrawingBackend, Size: SizeDesc> Drawable<DB> for Square<Coord, Size> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            let size = self.size.in_pixels(&ps);
            return backend.draw_rect(
                (x - size, y - size),
                (x + size, y + size),
                &self.style,
                self.style.filled,
            );
        }
        Ok(())
    }
}

pub struct Triangle<Coord, Size: SizeDesc> {
    center: Coord,
    size: Size,
    style: ShapeStyle,
}

impl<Coord, Size: SizeDesc> Triangle<Coord, Size> {
    pub fn new<T: Into<ShapeStyle>>(coord: Coord, size: Size, style: T) -> Self {
        Self {
            center: coord,
            size,
            style: style.into(),
        }
    }
}

impl<'a, Coord: 'a, Size: SizeDesc> PointCollection<'a, Coord> for &'a Triangle<Coord, Size> {
    type Point = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        std::iter::once(&self.center)
    }
}

impl<Coord, DB: DrawingBackend, Size: SizeDesc> Drawable<DB> for Triangle<Coord, Size> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        ps: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            let size = self.size.in_pixels(&ps);
            let points = [-90, -210, -330, -90]
                .iter()
                .map(|deg| f64::from(*deg) * std::f64::consts::PI / 180.0)
                .map(|rad| {
                    (
                        (rad.cos() * f64::from(size) + f64::from(x)).ceil() as i32,
                        (rad.sin() * f64::from(size) + f64::from(y)).ceil() as i32,
                    )
                });
            if self.style.filled {
                backend.fill_polygon(points.into_iter().take(3), &self.style)?;
            } else {
                backend.draw_path(points, &self.style)?;
            }
        }
        Ok(())
    }
}

fn line_bench<'a, DB, X, Y>(
    chart: &mut ChartContext<'a, DB, Cartesian2d<X, Y>>,
    benches: &[Bench],
    test: &str,
    title: Option<&str>,
    line_style: &'a LineStyle,
) where
    DB: DrawingBackend + 'a,
    X: Ranged<ValueType = i32>,
    Y: Ranged<ValueType = u32>,
{
    chart
        .draw_series(LineSeries::new(
            filter_benches(benches, test),
            line_style.color,
        ))
        .unwrap()
        .label(title.unwrap_or(test))
        .legend(move |(x, y): (i32, i32)| {
            EmptyElement::at((x, y))
                + PathElement::new(vec![(0, 0), (20, 0)], line_style.color)
                + line_style.decorator.decorate((10, 0), line_style.color)
        });

    chart
        .draw_series(
            filter_benches(benches, test)
                .map(|(x, y): (i32, u32)| line_style.decorator.decorate((x, y), line_style.color)),
        )
        .unwrap();
}

fn filter_benches<'a>(
    benches: &'a [Bench],
    test: &'a str,
) -> impl Iterator<Item = (i32, u32)> + 'a {
    benches.iter().filter_map(move |b| {
        if b.test == test {
            println!("Using bench: {:?}", b);
            field_index(b.field.as_ref()).map(|i| (i, b.avg as u32))
        } else {
            None
        }
    })
}

fn chart_limits<'a, 'b>(it: impl Iterator<Item = &'a Bench>, tests: &'a [&'b str]) -> (u32, u32) {
    it.filter(|bench| tests.iter().any(|t| &bench.test == t))
        .fold(None, |lim, bench| {
            let x = bench.avg as u32;
            match lim {
                None => Some((x, x)),
                Some((min, max)) => Some((cmp::min(min, x), cmp::max(max, x))),
            }
        })
        .map(|(min, max)| (log10_floor(min), log10_ceil(max)))
        .expect("No benchmark found")
}

fn log10_floor(mut x: u32) -> u32 {
    let mut result: u32 = 1;
    while x >= 10 {
        result = result.saturating_mul(10);
        x /= 10;
    }
    result
}

fn log10_ceil(mut x: u32) -> u32 {
    let mut result: u32 = 1;
    x = x.saturating_sub(1);
    while x >= 1 {
        result = result.saturating_mul(10);
        x /= 10;
    }
    result
}

fn field_index(field: &str) -> Option<i32> {
    match field {
        "gf008" => Some(0),
        "gf016" => Some(1),
        "gf032" => Some(2),
        "gf064" => Some(3),
        "gf128" => Some(4),
        "gf256" => Some(5),
        "gf512" => Some(6),
        "gf1024" => Some(7),
        "gf2048" => Some(8),
        _ => None,
    }
}

fn field_formatter(value: i32) -> &'static str {
    match value {
        0 => "GF(2^8)",
        1 => "GF(2^16)",
        2 => "GF(2^32)",
        3 => "GF(2^64)",
        4 => "GF(2^128)",
        5 => "GF(2^256)",
        6 => "GF(2^512)",
        7 => "GF(2^1024)",
        8 => "GF(2^2048)",
        _ => unreachable!(),
    }
}

fn parse(input: impl BufRead) -> Vec<Bench> {
    let re_bench =
        Regex::new(r"^test ([0-9a-z_]+)::test::([0-9a-z_]+)::([0-9a-z_:]+)\s+\.{3} bench:\s+([0-9,]+) ns/iter \(\+/\- ([0-9,]+)\)$").unwrap();

    let mut benches = Vec::new();
    for line in input.lines() {
        let line = line.unwrap();
        if let Some(caps) = re_bench.captures(&line) {
            println!("Line matches bench: {}", line);
            let field = caps[2].to_owned();
            let test = caps[3].to_owned();

            let mut avg = caps[4].to_owned();
            avg.retain(|c| c != ',');

            benches.push(Bench {
                field,
                test,
                avg: avg.parse().unwrap(),
            });
        }
    }

    benches.sort_by_key(|b| field_index(b.field.as_ref()));
    benches
}

#[derive(Debug)]
struct Bench {
    field: String,
    test: String,
    avg: u64,
}
