use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::prelude::SVGBackend;
use plotters::series::{LineSeries, PointSeries};
use plotters::style::{
    colors::{BLUE, RED},
    RGBColor, ShapeStyle,
};

fn main() {
    let width = 600;
    let height = 450;
    println!("Drawing area: {}, {}", width, height);

    // Basic Lagrange interpolation.
    let drawing_area = SVGBackend::new("lagrange.svg", (width, height)).into_drawing_area();
    let mut chart = ChartBuilder::on(&drawing_area)
        .x_label_area_size(20)
        .y_label_area_size(20)
        .margin(10)
        .build_cartesian_2d(0f32..8f32, 0f32..8f32)
        .unwrap();
    chart
        .configure_mesh()
        .x_labels(9)
        .y_labels(9)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.0}", v))
        .y_label_formatter(&|v| format!("{:.0}", v))
        .label_style(("sans-serif", 20))
        .draw()
        .unwrap();

    let polynom4 = |x| 4f32 + x * (-28f32 + x * (12f32 - x)) / 16f32;
    chart
        .draw_series(LineSeries::new(
            (0..=40).map(|i| {
                let x = i as f32 / 5f32;
                (x, polynom4(x))
            }),
            &BLUE,
        ))
        .unwrap();

    chart
        .draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            [(0f32, 4f32)],
            5,
            ShapeStyle::from(RGBColor(0, 0xC0, 0)).filled(),
        ))
        .unwrap();
    chart
        .draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            [
                (1f32, polynom4(1f32)),
                (2f32, polynom4(2f32)),
                (6f32, polynom4(6f32)),
                (8f32, polynom4(8f32)),
            ],
            5,
            ShapeStyle::from(&RED).filled(),
        ))
        .unwrap();
    chart
        .draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            [
                (3f32, polynom4(3f32)),
                (4f32, polynom4(4f32)),
                (5f32, polynom4(5f32)),
                (7f32, polynom4(7f32)),
            ],
            5,
            ShapeStyle::from(&BLUE).filled(),
        ))
        .unwrap();

    // Illustrate missing points.
    let drawing_area = SVGBackend::new("ambiguous.svg", (width, height)).into_drawing_area();
    let mut chart = ChartBuilder::on(&drawing_area)
        .x_label_area_size(20)
        .y_label_area_size(20)
        .margin(10)
        .build_cartesian_2d(0f32..8f32, 0f32..8f32)
        .unwrap();
    chart
        .configure_mesh()
        .x_labels(9)
        .y_labels(9)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.0}", v))
        .y_label_formatter(&|v| format!("{:.0}", v))
        .label_style(("sans-serif", 20))
        .draw()
        .unwrap();

    let polynom2 = |x| 2f32 + x * (-4f32 + x * (10f32 - x)) / 24f32;
    let polynom5 = |x| 5f32 + x * (-244f32 + x * (88f32 - x * 7f32)) / 96f32;
    let polynom7 = |x| 7f32 + x * (-132f32 + x * (40f32 - x * 3f32)) / 32f32;
    chart
        .draw_series(LineSeries::new(
            (0..=40).map(|i| {
                let x = i as f32 / 5f32;
                (x, polynom2(x))
            }),
            &RGBColor(0xFF, 0, 0xC0),
        ))
        .unwrap();
    chart
        .draw_series(LineSeries::new(
            (0..=40).map(|i| {
                let x = i as f32 / 5f32;
                (x, polynom4(x))
            }),
            &BLUE,
        ))
        .unwrap();
    chart
        .draw_series(LineSeries::new(
            (0..=40).map(|i| {
                let x = i as f32 / 5f32;
                (x, polynom5(x))
            }),
            &RGBColor(0, 0xC0, 0),
        ))
        .unwrap();
    chart
        .draw_series(LineSeries::new(
            (0..=40).map(|i| {
                let x = i as f32 / 5f32;
                (x, polynom7(x))
            }),
            &RGBColor(0xFF, 0x80, 0),
        ))
        .unwrap();

    chart
        .draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            [(0f32, 2f32), (0f32, 4f32), (0f32, 5f32), (0f32, 7f32)],
            5,
            ShapeStyle::from(RGBColor(0x80, 0x80, 0xC0)).filled(),
        ))
        .unwrap();
    chart
        .draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            [
                (2f32, polynom4(2f32)),
                (6f32, polynom4(6f32)),
                (8f32, polynom4(8f32)),
            ],
            5,
            ShapeStyle::from(&RED).filled(),
        ))
        .unwrap();
}
