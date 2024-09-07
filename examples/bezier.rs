use ori::prelude::*;

struct Data {
    points: Vec<Point>,
    selected: Option<usize>,
    cap: StrokeCap,
    join: StrokeJoin,
    cubic: bool,
    offset: f32,
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
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
            cubic: false,
            offset: 50.0,
        }
    }
}

fn ui(data: &mut Data) -> impl View<Data> {
    let curve = painter(|cx, data: &mut Data| {
        let styles = cx.styles();
        let primary: Color = styles.get("palette.primary").unwrap();
        let success: Color = styles.get("palette.success").unwrap();
        let accent: Color = styles.get("palette.accent").unwrap();

        let mut curve = Curve::new();

        curve.move_to(data.points[0]);

        if data.cubic {
            for p in data.points[1..].chunks(3) {
                curve.cubic_to(p[0], p[1], p[2]);
            }
        } else {
            for p in data.points[1..].chunks(2) {
                curve.quad_to(p[0], p[1]);
            }
        }

        cx.stroke(curve.clone(), Stroke::from(2.0), primary);

        let mut stroke = Curve::new();

        stroke.stroke_curve(
            &curve,
            Stroke {
                width: data.offset,
                cap: data.cap,
                join: data.join,
                miter: 4.0,
            },
        );

        cx.stroke(stroke, Stroke::from(1.0), success);

        for point in &data.points {
            let curve = Curve::circle(*point, 5.0);

            cx.stroke(curve, Stroke::from(2.0), accent);
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
                cx.draw();
                cx.rebuild();
            }
        }
        Event::PointerReleased(_) => {
            data.selected = None;
        }
        _ => {}
    });

    let offset_slider = hstack![
        text("Offset"),
        slider(data.offset)
            .range(0.0..=100.0)
            .on_input(|cx, data: &mut Data, offset| {
                data.offset = offset;
                cx.rebuild();
            }),
    ];

    let line_cap = on_click(button(text!("{:?}", data.cap)), |cx, data: &mut Data| {
        data.cap = match data.cap {
            StrokeCap::Butt => StrokeCap::Round,
            StrokeCap::Round => StrokeCap::Square,
            StrokeCap::Square => StrokeCap::Butt,
        };

        cx.rebuild();
    });

    let line_join = on_click(button(text!("{:?}", data.join)), |cx, data: &mut Data| {
        data.join = match data.join {
            StrokeJoin::Miter => StrokeJoin::Round,
            StrokeJoin::Round => StrokeJoin::Bevel,
            StrokeJoin::Bevel => StrokeJoin::Miter,
        };

        cx.rebuild();
    });

    let cubic = hstack![
        text("Cubic"),
        on_click(checkbox(data.cubic), |cx, data: &mut Data| {
            data.cubic = !data.cubic;
            cx.rebuild();
        }),
    ]
    .gap(10.0);

    center(
        vstack![
            size(400.0, curve),
            offset_slider,
            vstack![line_cap, line_join, cubic]
                .gap(10.0)
                .align(Align::Stretch),
        ]
        .gap(10.0),
    )
}

fn main() {
    ori::log::install().unwrap();

    let window = Window::new().title("Bezier (examples/bezier.rs)");

    let app = App::build().window(window, ui).theme(Theme::light());

    ori::run(app, &mut Data::new()).unwrap();
}
