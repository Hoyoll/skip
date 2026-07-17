use std::marker::PhantomData;

use skip::{Color, Div, Proc, Renderer, Vec2};

pub struct Border<Color: Into<Color>, Thickness: Into<Vec2<f32>>, Offset: Into<Vec2<f32>>>(pub Color, pub Thickness, pub Offset);

impl<Color: Into<Color>, Thickness: Into<Vec2<f32>>, Offset: Into<Vec2<f32>>> Border<Color, Thickness, Offset> {
    
}

impl<'skip,R: Renderer, C: Into<Color>, Thickness: Into<Vec2<f32>>, Offset: Into<Vec2<f32>>> Proc<'skip, R> for Border<C, Thickness, Offset> {
    type Arg = ();
    type Widget = Div<R>;
    fn consume(self, widget: Self::Widget, _argv: Self::Arg) -> Self::Widget {
        let col = &self.0.into();
        let large = &self.1.into();
        let pad = &self.2.into();
        widget
            .child(|div: Div<_>| {
                div
                    .color(col)
                    .enlarge(large)
                    .padding(pad)
                    .render()
            })
    }
}

fn main() {
    println!("Hello, world!");
}
