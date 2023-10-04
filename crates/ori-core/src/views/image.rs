use crate::{
    canvas::{Canvas, Color, Mesh, Vertex},
    event::Event,
    image::{Image, Texture},
    layout::{Point, Size, Space},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

impl<T> View<T> for Image {
    type State = ();

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {}

    fn rebuild(&mut self, _state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        if self != old {
            cx.request_layout();
            cx.request_draw();
        }
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.fit(self.size())
    }

    fn draw(
        &mut self,
        _state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        let mut mesh = Mesh::new();

        mesh.vertices.push(Vertex {
            position: cx.rect().top_left(),
            tex_coords: Point::new(0.0, 0.0),
            color: Color::WHITE,
        });
        mesh.vertices.push(Vertex {
            position: cx.rect().top_right(),
            tex_coords: Point::new(1.0, 0.0),
            color: Color::WHITE,
        });
        mesh.vertices.push(Vertex {
            position: cx.rect().bottom_right(),
            tex_coords: Point::new(1.0, 1.0),
            color: Color::WHITE,
        });
        mesh.vertices.push(Vertex {
            position: cx.rect().bottom_left(),
            tex_coords: Point::new(0.0, 1.0),
            color: Color::WHITE,
        });

        mesh.indices.push(0);
        mesh.indices.push(1);
        mesh.indices.push(2);
        mesh.indices.push(0);
        mesh.indices.push(2);
        mesh.indices.push(3);

        mesh.texture = Some(Texture::Image(self.clone()));

        canvas.draw(mesh);
    }
}
