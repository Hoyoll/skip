pub mod cn;

mod builtin;

use std::marker::PhantomData;

use crate::{builtin::Widget, child::{Child, Horizontal, Manual, Vertical}, io::{Mouse, State}, op::Operation, size::Size};
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

trait COption: Default {
    type Data;
    //fn new() -> impl COption;
    fn as_ref(&self, _f: impl FnOnce(&Self::Data)) {}
    fn as_mut(&mut self, _f: impl FnOnce(&mut Self::Data)) {}
    fn take(self, f: impl FnOnce(Self::Data));
    fn take_back(self, f: impl FnOnce(Self::Data) -> Self::Data);
    //fn put(&mut self, data: T);
}

#[derive(Default, Debug)]
pub struct CSome<T: Default>(pub T);

impl<T: Default> COption for CSome<T> {
    type Data = T;

    fn as_ref(&self, f: impl FnOnce(&T)) {
        f(&self.0)
    }

    fn as_mut(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.0)
    }

    fn take(self, f: impl FnOnce(T)) {
        f(self.0)
    }

    fn take_back(mut self, f: impl FnOnce(T) -> T) {
        self.0 = f(self.0);
    }
}

//#[derive(Default)]
pub struct CNone<T>(PhantomData<T>);

impl<T> CNone<T> {
    pub fn new() -> Self {
        Self(PhantomData::default())
    }
}

impl<T> Default for CNone<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> COption for CNone<T> {
    type Data = T;
    fn take(self, _f: impl FnOnce(Self::Data)) {}
    fn take_back(self, _f: impl FnOnce(Self::Data) -> Self::Data) {}
}

pub struct Circle<R: Renderer> {
    widget: CircleW,
    renderer: R,
}

pub struct CircleW {
    pub radius: f32,
    pub pos: Vec2<f32>,
}
//struct TextMetrics {
//    ascent: f32,
//    descent: f32,
//    line_height: f32,
//}

//cursor_height = metrics.ascent + metrics.descent; tight
//cursor_height = metrics.line_height; span

pub struct Text<'skip, R: Renderer, TD: TextD<'skip, R> = Empty> {
    widget: TextW,
    content: TD::Text,
    renderer: R,    
}

pub struct TextW {
    pub dim: Vec2<f32>,
    pub pos: Vec2<f32>, 
}

pub trait TextD<'skip, R: Renderer> {
    type Text: Default;
    fn measure(
        renderer: &mut R,
        text_w: &mut TextW,
        text: Self::Text, 
    ) -> Self::Text;
    //fn clone(text: Self::Text) -> Self::Text;
    fn display(
        renderer: &mut R,
        widget: &TextW,
        text: Self::Text, 
        color: Color,
    ) -> Self::Text;
}
pub struct Empty;

impl<'skip, R: Renderer> TextD<'skip, R> for Empty {
    type Text = ();
    fn measure(
        _renderer: &mut R,
        _text_w: &mut TextW,
        text: Self::Text,
    ) -> Self::Text {
        text
    }

    fn display(
        _renderer: &mut R,
        _widget: &TextW,
        text: Self::Text,
        _color: Color,
    ) -> Self::Text {
        text
    }
}

pub struct Wrap;
pub struct Paragraph<R: Renderer> {
    pub text: String,
    pub cached_paragraph: Option<R::Paragraph>,
}
impl<'skip, R: Renderer + 'skip> TextD<'skip, R> for Wrap {
    type Text = Option<(&'skip mut Paragraph<R>, &'skip R::Font)>;
    fn measure(
        renderer: &mut R,
        text_w: &mut TextW,
        text: Self::Text,
        //font: &<R as Renderer>::Font,
    ) -> Self::Text {
        text.map(|(text, font)| {
            renderer.measure_paragraph(text_w, &text.text, &mut text.cached_paragraph, font);
            (text, font)
        })
    }

    fn display(
        renderer: &mut R,
        widget: &TextW,
        text: Self::Text,
        //font: &<R as Renderer>::Font,
        color: Color,
    ) -> Self::Text {
        text.map(|(text, font)| {
            renderer.render_paragraph(widget, &text.text, &mut text.cached_paragraph, font, color);
            (text, font)
        })
    }
}

pub struct Linear;

impl<'skip, R: Renderer + 'skip> TextD<'skip, R> for Linear {
    type Text = Option<(&'skip str, &'skip R::Font)>;
    fn measure(
        renderer: &mut R,
        text_w: &mut TextW,
        text: Self::Text,
        //font: &<R as Renderer>::Font,
    ) -> Self::Text {
        text.map(|(text, font)| {
            renderer.measure_text(text_w, text, font);
            (text, font)
        })
    }

    fn display(
        renderer: &mut R,
        widget: &TextW,
        text: Self::Text,
        //font: &<R as Renderer>::Font,
        color: Color,
    ) -> Self::Text {
        text.map(|(text, font)| {
            renderer.render_text(widget, text, font, color);
            (text, font)
        })
    }
}

pub struct Div<R: Renderer, L: Child = ()> {
    widget: DivW,
    layout: L,
    renderer: R,
}

pub mod child {
    use crate::{COption, Renderer, Vec2, builtin::Widget};

    pub trait Child: Default {
        fn layout<'skip, W: Widget<'skip, R>, WO: Widget<'skip, R>, R: Renderer>(
            &mut self,
            renderer: R,
            //_arg: W::Constructor,
            _parent: &Vec2<f32>,
            _f: impl FnOnce(W) -> WO,
        ) -> R {
            renderer
        }
        fn iter<'skip, R: Renderer, Iter: Iterator, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
            &mut self,
            renderer: R,
            //_arg: W::Constructor,
            _parent: &Vec2<f32>,
            _items: (Iter, impl COption<Data = usize>),
            _f: impl FnMut(W, Iter::Item) -> WO,
        ) -> R {
            renderer
        }
    }

    impl Child for () {}

    #[derive(Default, Debug)]
    pub struct Manual;

    #[derive(Default)]
    pub struct Vertical {
        offset: Vec2<f32>,
        gap: f32,
    }

    #[derive(Default)]
    pub struct Horizontal {
        offset: Vec2<f32>,
        gap: f32,
    }

    impl Child for Manual {
        fn layout<'skip, W: Widget<'skip, R>, WO: Widget<'skip, R>, R: Renderer>(
            &mut self,
            renderer: R,
            //arg: W::Constructor,
            parent: &Vec2<f32>,
            f: impl FnOnce(W) -> WO,
        ) -> R {
            f(W::inherit(parent, renderer)).renderer()
        }
    }

    impl Child for Vertical {
        fn layout<'skip, W: Widget<'skip, R>, WO: Widget<'skip, R>, R: Renderer>(
            &mut self,
            renderer: R,
            //arg: W::Constructor,
            parent: &Vec2<f32>,
            f: impl FnOnce(W) -> WO,
        ) -> R {
            let w = f(W::inherit(
                (parent.x, parent.y + self.offset.y),
                //arg,
                renderer,
            ));
            let size = w.size();
            self.offset.y += size.y;
            self.offset.y += self.gap;
            w.renderer()
        }

        fn iter<'skip, R: Renderer, Iter: Iterator, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
            &mut self,
            renderer: R,
            //arg: W::Constructor,
            parent: &Vec2<f32>,
            (items, column): (Iter, impl COption<Data = usize>),
            mut f: impl FnMut(W, Iter::Item) -> WO,
        ) -> R {
            let mut w;
            let mut iter = items;
            let mut renderer = renderer;
            let mut limit = 0;
            for i in iter.by_ref() {
                w = f(
                    W::inherit(
                        (parent.x + self.offset.x, parent.y + self.offset.y),
                        //arg.clone(),
                        renderer,
                    ),
                    i,
                );
                let size = w.size();
                self.offset.y += size.y;
                self.offset.y += self.gap;
                renderer = w.renderer();
                column.as_ref(|column| {
                    limit += 1;
                    if *column == limit {
                        limit = 0;
                        self.offset.y = 0.0;
                        self.offset.x += size.x + self.gap;
                    }
                });
            }
            renderer
        }
    }

    impl Child for Horizontal {
        fn layout<'skip, W: Widget<'skip, R>, WO: Widget<'skip, R>, R: Renderer>(
            &mut self,
            renderer: R,
            //arg: W::Constructor,
            parent: &Vec2<f32>,
            f: impl FnOnce(W) -> WO,
        ) -> R {
            let w = f(W::inherit(
                (parent.x + self.offset.x, parent.y),
                //arg,
                renderer,
            ));
            let size = w.size();
            self.offset.x += size.x;
            self.offset.x += self.gap;
            w.renderer()
        }

        fn iter<'skip, R: Renderer, Iter: Iterator, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
            &mut self,
            renderer: R,
            //arg: W::Constructor,
            parent: &Vec2<f32>,
            (items, column): (Iter, impl COption<Data = usize>),
            //items: impl Into<IterArg<Iter>>,
            mut f: impl FnMut(W, Iter::Item) -> WO,
        ) -> R {
            let mut w;
            let mut iter = items;
            let mut renderer = renderer;
            let mut limit = 0;
            for item in iter.by_ref() {
                w = f(
                    W::inherit(
                        (parent.x + self.offset.x, parent.y + self.offset.y),
                        //arg.clone(),
                        renderer,
                    ),
                    item,
                );
                let size = w.size();
                self.offset.x += size.x;
                self.offset.x += self.gap;
                renderer = w.renderer();
                column.as_ref(|column| {
                    limit += 1;
                    if *column == limit {
                        limit = 0;
                        self.offset.x = 0.0;
                        self.offset.y += size.y + self.gap;
                    }
                });
            }
            renderer
        }
    }
}

pub mod size {
    use crate::{Renderer, Vec2};
    trait PSize {}
    pub trait Size<R: Renderer>: PSize {
        fn get(self, renderer: &R) -> Vec2<f32>;
    }

    pub struct Screen;
    impl PSize for Screen {}
    impl<R: Renderer> Size<R> for Screen {
        fn get(self, renderer: &R) -> Vec2<f32> {
            renderer.canvas_size()
        }
    }

    pub struct Inherit;
    impl PSize for Inherit {}
    impl<R: Renderer> Size<R> for Inherit {
        fn get(self, renderer: &R) -> Vec2<f32> {
            renderer.get_parent().0
        }
    }

    impl PSize for (f32, f32) {}
    impl<R: Renderer> Size<R> for (f32, f32) {
        fn get(self, _renderer: &R) -> Vec2<f32> {
            self.into()
        }
    }

    impl PSize for Vec2<f32> {}
    impl<R: Renderer> Size<R> for Vec2<f32> {
        fn get(self, _renderer: &R) -> Vec2<f32> {
            self
        }
    }
}

pub struct DivW {
    pub size: Vec2<f32>,
    pub pos: Vec2<f32>,
}

impl<'skip, R: Renderer> Circle<R> {
    #[inline]
    pub fn expr<T>(self, (expr_pack, f): (T, impl FnOnce(Self, T) -> Self)) -> Self {
        f(self, expr_pack)
    }

    #[inline]
    pub fn radius(mut self, rad: f32) -> Self {
        self.widget.radius = rad;
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.widget.pos, pos.into());
        self
    }
    #[inline]
    pub fn render(mut self, color: impl Into<Color>) -> Self {
        self.renderer.render_circle(&self.widget, color.into());
        self
    }

    #[inline]
    pub fn proc<P: Proc<'skip, R, Widget = Self>>(self, proc: P, arg: P::Arg) -> Self {
        proc.consume(self, arg)
    }
}

impl<R: Renderer> Div<R, Vertical> {
    #[inline]
    pub fn child<'skip, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
        mut self,
        //arg: impl Into<W::Constructor>,
        f: impl FnOnce(W) -> WO,
    ) -> Self {
        self.renderer
            .set_parent(&self.widget.size, &self.widget.pos);
        self.renderer = self.layout.layout(self.renderer, &self.widget.pos, f);
        self
    }

    #[inline]
    pub fn iter<'skip, Iter: Iterator, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
        mut self,
        //arg: impl Into<W::Constructor>,
        items: (Iter, impl COption<Data = usize>),
        f: impl FnMut(W, Iter::Item) -> WO,
    ) -> Self {
        self.renderer = self.layout.iter(self.renderer, &self.widget.pos, items, f);
        self
    }
}

impl<R: Renderer> Div<R, Horizontal> {
    #[inline]
    pub fn child<'skip, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
        mut self,
        //arg: impl Into<W::Constructor>,
        f: impl FnOnce(W) -> WO,
    ) -> Self {
        self.renderer
            .set_parent(&self.widget.size, &self.widget.pos);
        self.renderer = self.layout.layout(self.renderer, &self.widget.pos, f);
        self
    }

    #[inline]
    pub fn iter<'skip, Iter: Iterator, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
        mut self,
        //arg: impl Into<W::Constructor>,
        items: (Iter, impl COption<Data = usize>),
        f: impl FnMut(W, Iter::Item) -> WO,
    ) -> Self {
        self.renderer = self.layout.iter(self.renderer, &self.widget.pos, items, f);
        self
    }
}

impl<R: Renderer> Div<R, Manual> {
        #[inline]
    pub fn child<'skip, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
        mut self,
        //arg: impl Into<W::Constructor>,
        f: impl FnOnce(W) -> WO,
    ) -> Self {
        self.renderer
            .set_parent(&self.widget.size, &self.widget.pos);
        self.renderer = self.layout.layout(self.renderer, &self.widget.pos, f);
        self
    }

    #[inline]
    pub fn iter<'skip, Iter: Iterator, W: Widget<'skip, R>, WO: Widget<'skip, R>>(
        mut self,
        //arg: impl Into<W::Constructor>,
        items: (Iter, impl COption<Data = usize>),
        f: impl FnMut(W, Iter::Item) -> WO,
    ) -> Self {
        self.renderer = self.layout.iter(self.renderer, &self.widget.pos, items, f);
        self
    }
}

impl<R: Renderer, L: Child> Div<R, L> {
    #[inline]
    pub fn proc<'skip, P: Proc<'skip, R, Widget = Self>>(self, proc: P, arg: P::Arg) -> Self {
        proc.consume(self, arg)
    }

    #[inline]
    pub fn align<Align: align::Align, Apply: apply::Apply>(mut self) -> Self {
        let res = Align::calc(self.renderer.get_parent(), &self.widget.size);
        Apply::apply(res, &mut self.widget.pos);
        self
    }

    #[inline]
    pub fn render<'skip, Style: style::Style<'skip, R>>(mut self, arg: Style::Arg) -> Self {
        Style::render(&self.widget, &mut self.renderer, arg);
        self
    }

    #[inline]
    pub fn on<'skip, On: on::On<'skip, R, Self>>(
        mut self,
        f: impl FnMut(On::Arg) -> On::FnOut,
    ) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.widget.pos.x)
            && (mouse_pos.y >= self.widget.pos.y)
            && (mouse_pos.x <= (self.widget.pos.x + self.widget.size.x))
            && (mouse_pos.y <= (self.widget.pos.y + self.widget.size.y));
        if !hovered {
            return self;
        }
        On::call(f, self, mouse_pos)
    }

    #[inline]
    pub fn size<Op: Operation<Item = Vec2<f32>>>(mut self, size: impl Size<R>) -> Self {
        let s = size.get(&self.renderer);
        Op::apply(&mut self.widget.size, s);
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.widget.pos, pos.into());
        self
    }

    #[inline]
    pub fn expr<T>(self, (expr_pack, f): (T, impl FnOnce(Self, T) -> Self)) -> Self {
        f(self, expr_pack)
    }
}

impl<'skip, R: Renderer + 'skip> Text<'skip, R, Wrap> {
    pub fn render(mut self, color: impl Into<Color>) -> Self {
        //if let Some(text) = self.content {
        self.content = Wrap::display(&mut self.renderer, &self.widget, self.content, color.into());
        //}
        self
    }

    pub fn on<On: on::On<'skip, R, Self>>(
        mut self,
        f: impl FnMut(On::Arg) -> On::FnOut,
    ) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.widget.pos.x)
            && (mouse_pos.y >= self.widget.pos.y)
            && (mouse_pos.x <= (self.widget.pos.x + self.widget.dim.x))
            && (mouse_pos.y <= (self.widget.pos.y + self.widget.dim.y));
        if !hovered {
            return self;
        }
        On::call(f, self, mouse_pos)
    }
}

impl<'skip, R: Renderer + 'skip> Text<'skip, R, Linear> {
    pub fn render(mut self, color: impl Into<Color>) -> Self {
        //if let Some(text) = self.content {
        self.content =
            Linear::display(&mut self.renderer, &self.widget, self.content, color.into());
        //}
        self
    }

    pub fn on<On: on::On<'skip, R, Self>>(
        mut self,
        f: impl FnMut(On::Arg) -> On::FnOut,
    ) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.widget.pos.x)
            && (mouse_pos.y >= self.widget.pos.y)
            && (mouse_pos.x <= (self.widget.pos.x + self.widget.dim.x))
            && (mouse_pos.y <= (self.widget.pos.y + self.widget.dim.y));
        if !hovered {
            return self;
        }
        On::call(f, self, mouse_pos)
    }
}

impl<'skip, R: Renderer, TD: TextD<'skip, R>> Text<'skip, R, TD> {
    #[inline]
    pub fn expr<T, WO: Widget<'skip, R>>(
        self,
        (expr_pack, f): (T, impl FnOnce(Self, T) -> WO),
    ) -> WO {
        f(self, expr_pack)
    }

    #[inline]
    pub fn content<NewTD: TextD<'skip, R>>(self, content: NewTD::Text) -> Text<'skip, R, NewTD> {
        Text {
            widget: self.widget,
            content,
            renderer: self.renderer,
        }
    }

    #[inline]
    pub fn proc<P: Proc<'skip, R, Widget = Self>>(self, proc: P, arg: P::Arg) -> Self {
        proc.consume(self, arg)
    }

    pub fn align<Align: align::Align, Apply: apply::Apply>(mut self) -> Self {
        let res = Align::calc(self.renderer.get_parent(), &self.widget.dim);
        Apply::apply(res, &mut self.widget.pos);
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.widget.pos, pos.into());
        self
    }
}

pub mod op {
    use crate::Vec2;

    pub trait Operation {
        type Item;

        fn apply(initial_item: &mut Self::Item, diff: Self::Item);
    }

    pub struct Inc;

    impl Operation for Inc {
        type Item = Vec2<f32>;

        fn apply(initial_item: &mut Self::Item, diff: Self::Item) {
            initial_item.x += diff.x;
            initial_item.y += diff.y;
        }
    }

    pub struct Set;

    impl Operation for Set {
        type Item = Vec2<f32>;

        fn apply(initial_item: &mut Self::Item, diff: Self::Item) {
            initial_item.x = diff.x;
            initial_item.y = diff.y;
        }
    }

    pub struct Dec;

    impl Operation for Dec {
        type Item = Vec2<f32>;

        fn apply(initial_item: &mut Self::Item, diff: Self::Item) {
            initial_item.x -= diff.x;
            initial_item.y -= diff.y;
        }
    }

    pub struct Divide;

    impl Operation for Divide {
        type Item = Vec2<f32>;

        fn apply(initial_item: &mut Self::Item, diff: Self::Item) {
            initial_item.x /= diff.x;
            initial_item.y /= diff.y;
        }
    }

    pub struct Mul;

    impl Operation for Mul {
        type Item = Vec2<f32>;

        fn apply(initial_item: &mut Self::Item, diff: Self::Item) {
            initial_item.x *= diff.x;
            initial_item.y *= diff.y;
        }
    }
}

pub mod on {
    use crate::{Renderer, Vec2, builtin::Widget, io::{Mouse, State}};

    pub trait On<'skip, R: Renderer, W: Widget<'skip, R>> {
        type FnOut;
        type Arg: 'skip;
        fn call(f: impl FnMut(Self::Arg) -> Self::FnOut, widget: W, cursor_pos: Vec2<f32>) -> W;
    }

    pub struct Hover;

    impl<'skip, R: Renderer, W: Widget<'skip, R> + 'skip> On<'skip, R, W> for Hover {
        type Arg = (Vec2<f32>, W);

        //type Out = W;
        type FnOut = W;

        fn call(
            mut f: impl FnMut(Self::Arg) -> Self::FnOut,
            widget: W,
            cursor_pos: Vec2<f32>,
        ) -> W {
            f((cursor_pos, widget))
        }
    }

    pub struct Mouses;

    impl<'skip, R: Renderer, W: Widget<'skip, R>> On<'skip, R, W> for Mouses {
        type FnOut = ();

        type Arg = &'skip (Mouse, State);

        fn call(f: impl FnMut(Self::Arg) -> Self::FnOut, widget: W, _: Vec2<f32>) -> W {
            widget.iter_mouse(f);
            widget
        }
    }
}

pub mod style {
    use crate::{Color, DivW, Renderer};

    pub trait Style<'skip, R: Renderer> {
        type Arg: 'skip;
        fn render(div: &DivW, renderer: &mut R, arg: Self::Arg);
    }

    pub struct Plain;

    pub struct Image;

    impl<'skip, R: Renderer> Style<'skip, R> for Plain {
        type Arg = (Color, f32);
        fn render(div: &DivW, renderer: &mut R, (color, rad): Self::Arg) {
            renderer.render_div(div, color, rad);
        }
    }

    impl<'skip, R: Renderer + 'skip> Style<'skip, R> for Image {
        type Arg = (&'skip R::Image, Color);
        fn render(div: &DivW, renderer: &mut R, (img, color): Self::Arg) {
            renderer.render_img(div, color, img);
            //        renderer.render_img(div, self.tint.into(), self.img_id);
        }
    }
}

pub mod align {
    use crate::Vec2;

    pub trait Align {
        fn calc(
            parent: impl Into<(Vec2<f32>, Vec2<f32>)>,
            child_dim: impl Into<Vec2<f32>>,
        ) -> Vec2<f32>;
    }

    pub struct Center;

    impl Align for Center {
        fn calc(
            parent: impl Into<(Vec2<f32>, Vec2<f32>)>,
            child_dim: impl Into<Vec2<f32>>,
        ) -> Vec2<f32> {
            let (p_size, p_pos) = parent.into();
            let size = child_dim.into();
            let center_pos: Vec2<_> = (p_pos.x + p_size.x / 2.0, p_pos.y + p_size.y / 2.0).into();
            (center_pos.x - size.x / 2.0, center_pos.y - size.y / 2.0).into()
        }
    }

    pub struct End;

    impl Align for End {
        fn calc(
            parent: impl Into<(Vec2<f32>, Vec2<f32>)>,
            child_dim: impl Into<Vec2<f32>>,
        ) -> Vec2<f32> {
            let (p_size, p_pos) = parent.into();
            let size = child_dim.into();
            let end_pos: Vec2<_> = (p_pos.x + p_size.x, p_pos.y + p_size.y).into();
            (end_pos.x - size.x, end_pos.y - size.y).into()
        }
    }

    pub struct Start;

    impl Align for Start {
        fn calc(
            parent: impl Into<(Vec2<f32>, Vec2<f32>)>,
            _child_dim: impl Into<Vec2<f32>>,
        ) -> Vec2<f32> {
            parent.into().1
        }
    }
}

pub mod apply {
    use crate::Vec2;

    pub trait Apply {
        fn apply(new_coord: Vec2<f32>, coord: &mut Vec2<f32>);
    }

    pub struct X;

    pub struct Y;

    pub struct XY;

    impl Apply for XY {
        fn apply(new_coord: Vec2<f32>, coord: &mut Vec2<f32>) {
            *coord = new_coord;
        }
    }

    impl Apply for X {
        fn apply(new_coord: Vec2<f32>, coord: &mut Vec2<f32>) {
            coord.x = new_coord.x;
        }
    }

    impl Apply for Y {
        fn apply(new_coord: Vec2<f32>, coord: &mut Vec2<f32>) {
            coord.y = new_coord.y;
        }
    }
}

pub trait Renderer {
    type Paragraph;
    type Image;
    type Font;
    fn render_div(&mut self, div: &DivW, color: Color, radius: f32);
    fn render_img(&mut self, img: &DivW, color: Color, image: &Self::Image);
    fn render_circle(&mut self, circle: &CircleW, color: Color);
    fn render_paragraph(
        &mut self,
        text_w: &TextW,
        text: &str,
        paragraph: &mut Option<Self::Paragraph>,
        font: &Self::Font,
        color: Color,
    );
    fn render_text(&mut self, text_w: &TextW, text: &str, font: &Self::Font, color: Color);
    fn measure_text(&mut self, text_w: &mut TextW, text: &str, font: &Self::Font);
    fn measure_paragraph(
        &mut self,
        text_w: &mut TextW,
        text: &str,
        paragraph: &mut Option<Self::Paragraph>,
        font: &Self::Font,
    );
    //fn text_size<'skip>(&mut self, text_w: &TextW<'skip>, text: &str) -> Vec2<f32>;
    //fn paragraph_size<'skip>(&mut self, text_w: &TextW<'skip>, text: &str, paragraph: &mut Option<Self::Paragraph>) -> Vec2<f32>;
    fn start_clip(&mut self, dim: &Vec2<f32>, pos: &Vec2<f32>);
    fn mouse_pos(&mut self) -> Vec2<f32>;
    //fn mouse_state(&mut self) -> &Vec<(Mouse, State)>;
    fn end_clip(&mut self);
    fn canvas_size(&self) -> Vec2<f32>;
    //fn change_cursor(&mut self, cursor: Cursor);
    fn set_parent<Dim: Into<Vec2<f32>>, Pos: Into<Vec2<f32>>>(&mut self, dim: Dim, pos: Pos);
    fn get_parent(&self) -> (Vec2<f32>, Vec2<f32>);
    fn iter_mouse<'a, F: FnMut(&'a (Mouse, State))>(&self, f: F);
}

pub trait Proc<'skip, R: Renderer> {
    type Widget: Widget<'skip, R>;
    type Arg;
    fn consume(self, widget: Self::Widget, argv: Self::Arg) -> Self::Widget;
}

impl<'skip, R: Renderer> Widget<'skip, R> for Circle<R> {
    fn inherit<PO: Into<Vec2<f32>>>(pos: PO, renderer: R) -> Self {
        //let radius = radius.as_ref();
        Self {
            widget: CircleW {
                radius: 0.0,
                pos: pos.into(),
            },
            renderer,
        }
    }
    fn renderer(self) -> R {
        self.renderer
    }
    fn size(&self) -> Vec2<f32> {
        (self.widget.radius, self.widget.radius).into()
    }

    fn iter_mouse<'a, F: FnMut(&'a (Mouse, State))>(&self, f: F) {
        self.renderer.iter_mouse(f);
    }
}

impl<'skip, R: Renderer, L: Child> Widget<'skip, R> for Div<R, L> {
    //type Constructor = Vec2<f32>;
    #[inline]
    fn inherit<PO: Into<Vec2<f32>>>(pos: PO, renderer: R) -> Self {
        let widget: DivW = DivW {
            size: ().into(),
            pos: pos.into(),
        };
        Self {
            widget,
            layout: L::default(),
            renderer,
        }
    }

    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }

    #[inline]
    fn size(&self) -> Vec2<f32> {
        (self.widget.size.x, self.widget.size.y).into()
    }
    fn iter_mouse<'a, F: FnMut(&'a (Mouse, State))>(&self, f: F) {
        self.renderer.iter_mouse(f);
    }
}

impl<'skip, R: Renderer, TD: TextD<'skip, R>> Widget<'skip, R> for Text<'skip, R, TD> {
    //type Constructor = &'skip R::Font;
    #[inline]
    fn inherit<PO: Into<Vec2<f32>>>(pos: PO, renderer: R) -> Self {
        Self {
            widget: TextW {
                dim: ().into(),
                pos: pos.into(),
            },
            content: TD::Text::default(),
            //font: font,
            renderer,
            //ph: PhantomData::default(),
        }
    }

    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }
    #[inline]
    fn size(&self) -> Vec2<f32> {
        //self.renderer.text_size(&self.widget)

        (&self.widget.dim).into()
    }

    fn iter_mouse<'a, F: FnMut(&'a (Mouse, State))>(&self, f: F) {
        self.renderer.iter_mouse(f);
    }
}

pub mod io {
    pub enum State {
        Pressed,
        Released,
    }

    pub enum Mouse {
        Left,
        Right,
        Middle,
        Unknown,
    }

    pub enum Key {
        Num(&'static str),
        Char(&'static str),
        Named(&'static str),
        Symbol(&'static str),
        Unknown,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> AsRef<Vec2<T>> for Vec2<T> {
    #[inline]
    fn as_ref(&self) -> &Vec2<T> {
        self
    }
}

impl<T> Vec2<T> {
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> From<(T, T)> for Vec2<T> {
    #[inline]
    fn from(value: (T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl<T: Copy> From<(&Vec2<T>)> for Vec2<T> {
    #[inline]
    fn from(value: (&Vec2<T>)) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl<T: Default> From<()> for Vec2<T> {
    #[inline]
    fn from(_value: ()) -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    #[inline]
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self {
            r: value.0,
            g: value.1,
            b: value.2,
            a: value.3,
        }
    }
}

impl From<()> for Color {
    #[inline]
    fn from(_value: ()) -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

impl From<&Color> for Color {
    fn from(value: &Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl From<&mut Color> for Color {
    fn from(value: &mut Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}
