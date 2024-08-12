use ori::prelude::*;

struct Data {
    points: Vec<Point>,
    selected: Option<usize>,
    offset: f32,
    t: f32,
}

impl Data {
    fn new() -> Self {
        Self {
            points: vec![
                Point::new(100.0, 100.0),
                Point::new(150.0, 200.0),
                Point::new(200.0, 200.0),
                Point::new(250.0, 100.0),
                Point::new(300.0, 100.0),
                Point::new(350.0, 200.0),
                Point::new(400.0, 200.0),
            ],
            selected: None,
            offset: 10.0,
            t: 0.0,
        }
    }
}

fn ui(data: &mut Data) -> impl View<Data> {
    let curve = painter(|cx, data: &mut Data| {
        let mut curve = Curve::new();

        curve.move_to(data.points[0]);

        for p in data.points[1..].chunks(2) {
            curve.quad_to(p[0], p[1]);
        }

        let palette = palette();

        cx.stroke(curve.clone(), Stroke::from(2.0), palette.primary);

        let mut stroke = Curve::new();

        stroke.stroke_curve(
            &curve,
            Stroke {
                width: data.offset,
                cap: LineCap::Round,
                join: LineJoin::Round,
                miter: 4.0,
            },
        );

        let mut stroked = Curve::new();

        stroked.stroke_curve(
            &stroke,
            Stroke {
                width: 10.0,
                cap: LineCap::Round,
                join: LineJoin::Miter,
                miter: 4.0,
            },
        );

        cx.stroke(stroke, Stroke::from(1.0), palette.success);

        for point in &data.points {
            let curve = Curve::circle(*point, 5.0);

            cx.stroke(curve, Stroke::from(2.0), palette.accent);
        }
    });

    let curve = on_event(curve, |cx, data: &mut Data, event| match event {
        Event::PointerPressed(e) => {
            let local = cx.local(e.position);

            data.selected = None;

            for (i, point) in data.points.iter().enumerate() {
                if point.distance(local) < 10.0 {
                    data.selected = Some(i);
                    break;
                }
            }
        }
        Event::PointerMoved(e) => {
            let local = cx.local(e.position);

            if let Some(i) = data.selected {
                data.points[i] = local;
                cx.request_draw();
                cx.request_rebuild();
            }
        }
        Event::PointerReleased(_) => {
            data.selected = None;
        }
        _ => {}
    });

    let t_slider = hstack![
        text("T"),
        slider(data.t).on_input(|_, data: &mut Data, t| data.t = t),
    ];

    let offset_slider = hstack![
        text("Offset"),
        slider(data.offset)
            .range(0.0..=50.0)
            .on_input(|_, data: &mut Data, offset| data.offset = offset),
    ];

    center(vstack![size(400.0, curve), t_slider, offset_slider])
}

fn main() {
    let window = Window::new().title("Bezier Test (examples/bezier_test.rs)");

    let app = App::build().window(window, ui).style(Palette::light());

    ori::launch(app, Data::new()).unwrap();
}
