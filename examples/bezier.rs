use ori::prelude::*;

struct Data {
    points: Vec<Point>,
    selected: Option<usize>,
    line_cap: LineCap,
    line_join: LineJoin,
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
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            cubic: false,
            offset: 50.0,
        }
    }
}

fn ui(data: &mut Data) -> impl View<Data> {
    let curve = painter(|cx, data: &mut Data| {
        let palette = palette();

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

        cx.stroke(curve.clone(), Stroke::from(2.0), palette.primary);

        let mut stroke = Curve::new();

        stroke.stroke_curve(
            &curve,
            Stroke {
                width: data.offset,
                cap: data.line_cap,
                join: data.line_join,
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

    let offset_slider = hstack![
        text("Offset"),
        slider(data.offset)
            .range(0.0..=100.0)
            .on_input(|cx, data: &mut Data, offset| {
                data.offset = offset;
                cx.request_rebuild();
            }),
    ];

    let line_cap = on_click(
        button(text!("{:?}", data.line_cap)),
        |cx, data: &mut Data| {
            data.line_cap = match data.line_cap {
                LineCap::Butt => LineCap::Round,
                LineCap::Round => LineCap::Square,
                LineCap::Square => LineCap::Butt,
            };

            cx.request_rebuild();
        },
    );

    let line_join = on_click(
        button(text!("{:?}", data.line_join)),
        |cx, data: &mut Data| {
            data.line_join = match data.line_join {
                LineJoin::Miter => LineJoin::Round,
                LineJoin::Round => LineJoin::Bevel,
                LineJoin::Bevel => LineJoin::Miter,
            };

            cx.request_rebuild();
        },
    );

    let cubic = hstack![
        text("Cubic"),
        on_click(checkbox(data.cubic), |cx, data: &mut Data| {
            data.cubic = !data.cubic;
            cx.request_rebuild();
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
    let window = Window::new().title("Bezier (examples/bezier.rs)");

    let app = App::build().window(window, ui).style(Palette::light());

    ori::launch(app, Data::new()).unwrap();
}
