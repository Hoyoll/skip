use crate::{
    Color,Div, Horizontal, Inc, Inherit, Linear, Overflow, Plain, Proc, Renderer, Set, Text, Vec2
};

pub struct Border<Color: Into<crate::Color>, Thickness: Into<Vec2<f32>>, Offset: Into<Vec2<f32>>>(
    pub Color,
    pub Thickness,
    pub Offset,
);

impl<
    'skip,
    R: crate::Renderer,
    Color: Into<crate::Color>,
    Thickness: Into<Vec2<f32>>,
    Offset: Into<Vec2<f32>>,
> Proc<'skip, R> for Border<Color, Thickness, Offset>
{
    type Arg = ();
    type Widget = Div<R>;
    fn consume(self, widget: Self::Widget, _argv: Self::Arg) -> Self::Widget {
        let large = self.1.into();
        let mut pad = self.2.into();
        pad.x -= large.x;
        pad.y -= large.y;
        widget.child(Overflow,|div: Div<_>| {
            div
            .size::<Set>(Inherit)
            .size::<Inc>((large.x * 2.0, large.y * 2.0))
            .position::<Inc>(pad)
            .render::<Plain<_>>(self.0)
        })
    }
}

pub struct TextBox {
    pub text: String,
    pub insert_idx: usize,
}

impl TextBox {
    pub fn insert(&mut self, word: &str) {
        self.text.insert_str(self.insert_idx, word);
    }

    pub fn shift_left(&mut self) {
        if self.insert_idx == 0 {
            return;
        }
        self.insert_idx -= 1;
    }

    pub fn shift_right(&mut self) {
        self.insert_idx += 1;
    }
}

impl<'skip, R: Renderer + 'skip> Proc<'skip, R> for &'skip mut TextBox {
    type Widget = Horizontal<R>;
    type Arg = (&'skip R::Font, f32, Color);
    fn consume(self, widget: Self::Widget, (font, size, color): Self::Arg) -> Self::Widget {
        widget
            .add(|text: Text<Linear, _>| {
                text
                    .content((&self.text[0..self.insert_idx],font))
                    .size(size)
                    .render(&color)
            })
            .add(|cursor: Div<_>| {
                cursor
                    .size::<Set>((1.0, size)) 
                    .render::<Plain<_>>(&color)
            })
            .add(|text: Text<Linear, _>| {
                text
                    .content((&self.text[self.insert_idx..self.text.len()], font))
                    .size(size)
                    .render(&color)
            })
    }
}
