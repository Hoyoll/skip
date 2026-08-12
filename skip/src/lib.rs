pub mod cn;

mod builtin;

use std::marker::PhantomData;

use crate::builtin::{ProcArg, Widget};
#[derive(Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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


pub struct Text<'skip, TD: TextD<'skip, R>, R: Renderer> {
    widget: TextW,
    content: Option<(TD::Text, &'skip R::Font)>,
    renderer: R,
}

pub struct TextW {
    pub dim: Vec2<f32>,
    pub pos: Vec2<f32>,
    pub size: f32,
}

pub trait TextD<'skip, R: Renderer> {
    type Text: 'skip;
    fn measure(renderer: &mut R, text_w: &mut TextW, text: Self::Text, font: &R::Font) -> Self::Text;

    fn display(
        renderer: &mut R,
        widget: &TextW,
        text: Self::Text,
        font: &R::Font,
        color: Color,
    ) -> Self::Text;
}

pub struct Wrap;
pub struct Paragraph<R: Renderer> {
    text: String,
    cached_paragraph: Option<R::Paragraph>,
}
impl<'skip, R: Renderer + 'skip> TextD<'skip, R> for Wrap {
    type Text = &'skip mut Paragraph<R>;
    fn measure(renderer: &mut R, text_w: &mut TextW, text: Self::Text, font: &<R as Renderer>::Font) -> Self::Text {
        renderer.measure_paragraph(text_w, &text.text, &mut text.cached_paragraph, font);
        text
    }

    fn display(
        renderer: &mut R,
        widget: &TextW,
        text: Self::Text,
        font: &<R as Renderer>::Font,
        color: Color,
    ) -> Self::Text {
        renderer.render_paragraph(widget, &text.text, &mut text.cached_paragraph, font, color);
        text
    }
}

pub struct Linear;

impl<'skip, R: Renderer> TextD<'skip, R> for Linear {
    type Text = &'skip str;
    fn measure(renderer: &mut R, text_w: &mut TextW, text: Self::Text, font: &<R as Renderer>::Font) -> Self::Text {
        renderer.measure_text(text_w, text, font);
        text
    }
    fn display(
        renderer: &mut R,
        widget: &TextW,
        text: Self::Text,
        font: &<R as Renderer>::Font,
        color: Color,
    ) -> Self::Text {
        renderer.render_text(widget, text, font, color);
        text
    }
}

pub struct Div<L: LayoutRule, R: Renderer> {
    widget: DivW,
    layout: L,
    renderer: R,
}

pub trait LayoutRule: Default {
    fn layout<'skip, W: Widget<'skip, R>, R: Renderer>(
        &mut self,
        renderer: R,
        parent: &Vec2<f32>,
        f: impl FnOnce(W) -> W,
    ) -> R;
    fn iter<'skip, R: Renderer, Iter: Iterator, W: Widget<'skip, R>>(
        &mut self,
        renderer: R,
        parent: &Vec2<f32>,
        items: impl Into<IterArg<Iter>>,
        f: impl FnMut(W, Iter::Item) -> W,
    ) -> R;
}

impl LayoutRule for () {
    fn layout<'skip, W: Widget<'skip, R>, R: Renderer>(
        &mut self,
        renderer: R,
        parent: &Vec2<f32>,
        f: impl FnOnce(W) -> W,
    ) -> R {
        let w = f(W::inherit(parent, renderer));
        w.renderer()
    }

    fn iter<'skip, R: Renderer, Iter: Iterator, W: Widget<'skip, R>>(
        &mut self,
        renderer: R,
        _parent: &Vec2<f32>,
        _items: impl Into<IterArg<Iter>>,
        _f: impl FnMut(W, Iter::Item) -> W,
    ) -> R {
        renderer
    }
}

#[derive(Default)]
pub struct Vertical {
    offset: Vec2<f32>,
    gap: f32,
}

impl LayoutRule for Vertical {
    fn layout<'skip, W: Widget<'skip, R>, R: Renderer>(
        &mut self,
        renderer: R,
        parent: &Vec2<f32>,
        f: impl FnOnce(W) -> W,
    ) -> R {
        let w = f(W::inherit((parent.x, parent.y + self.offset.y), renderer));
        let size = w.size();
        self.offset.y += size.y;
        self.offset.y += self.gap;
        w.renderer()
    }

    fn iter<'skip, R: Renderer, Iter: Iterator, W: Widget<'skip, R>>(
        &mut self,
        renderer: R,
        parent: &Vec2<f32>,
        items: impl Into<IterArg<Iter>>,
        mut f: impl FnMut(W, Iter::Item) -> W,
    ) -> R {
        let mut w;
        let mut iter_arg = items.into();
        let mut renderer = renderer;
        match iter_arg.column {
            None => {
                for item in iter_arg.items.by_ref() {
                    w = f(
                        W::inherit((parent.x, parent.y + self.offset.y), renderer),
                        item,
                    );
                    let size = w.size();
                    self.offset.y += size.y;
                    self.offset.y += self.gap;
                    renderer = w.renderer();
                }
                return renderer;
            }
            Some(column) => {
                let mut limit: usize = 0;
                for item in iter_arg.items.by_ref() {
                    w = f(
                        W::inherit(
                            (parent.x + self.offset.x, parent.y + self.offset.y),
                            renderer,
                        ),
                        item,
                    );
                    let size = w.size();
                    self.offset.y += size.y;
                    self.offset.y += self.gap;
                    renderer = w.renderer();

                    limit += 1;
                    if column == limit {
                        limit = 0;
                        self.offset.y = 0.0;
                        self.offset.x += size.x + self.gap;
                    }
                }
                return renderer;
            }
        }
    }
}

#[derive(Default)]
pub struct Horizontal {
    offset: Vec2<f32>,
    gap: f32,
}

impl LayoutRule for Horizontal {
    fn layout<'skip, W: Widget<'skip, R>, R: Renderer>(
        &mut self,
        renderer: R,
        parent: &Vec2<f32>,
        mut f: impl FnOnce(W) -> W,
    ) -> R {
        let w = f(W::inherit((parent.x + self.offset.x, parent.y), renderer));
        let size = w.size();
        self.offset.x += size.x;
        self.offset.x += self.gap;
        w.renderer()
    }

    fn iter<'skip, R: Renderer, Iter: Iterator, W: Widget<'skip, R>>(
        &mut self,
        renderer: R,
        parent: &Vec2<f32>,
        items: impl Into<IterArg<Iter>>,
        mut f: impl FnMut(W, Iter::Item) -> W,
    ) -> R {
        let mut w;
        let mut iter_arg = items.into();
        let mut renderer = renderer;
        match iter_arg.column {
            None => {
                for item in iter_arg.items.by_ref() {
                    w = f(
                        W::inherit((parent.x + self.offset.x, parent.y), renderer),
                        item,
                    );
                    let size = w.size();
                    self.offset.x += size.x;
                    self.offset.x += self.gap;
                    renderer = w.renderer();
                }
                return renderer;
            }
            Some(column) => {
                let mut limit: usize = 0;
                for item in iter_arg.items.by_ref() {
                    w = f(
                        W::inherit(
                            (parent.x + self.offset.x, parent.y + self.offset.y),
                            renderer,
                        ),
                        item,
                    );
                    let size = w.size();
                    self.offset.x += size.x;
                    self.offset.x += self.gap;
                    renderer = w.renderer();

                    limit += 1;
                    if column == limit {
                        limit = 0;
                        self.offset.x = 0.0;
                        self.offset.y += size.y + self.gap;
                    }
                }
                return renderer;
            }
        }
    }
}

trait Size<R: Renderer> {
    fn get(self, renderer: &R) -> Vec2<f32>;
}

pub struct Screen;

impl<R: Renderer> Size<R> for Screen {
    fn get(self, renderer: &R) -> Vec2<f32> {
        renderer.canvas_size()
    }
}

pub struct Inherit;

impl<R: Renderer> Size<R> for Inherit {
    fn get(self, renderer: &R) -> Vec2<f32> {
        renderer.get_parent().0
    }
}

impl<R: Renderer> Size<R> for (f32, f32) {
    fn get(self, _renderer: &R) -> Vec2<f32> {
        self.into()
    }
}

impl<R: Renderer> Size<R> for Vec2<f32> {
    fn get(self, _renderer: &R) -> Vec2<f32> {
        self
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
    pub fn proc<PA: Into<ProcArg<'skip, R, P>>, P: Proc<'skip, R, Widget = Self>>(
        self,
        proc: PA,
    ) -> Self {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }
}

impl<'skip, R: Renderer, L: LayoutRule> Div<L, R> {
    #[inline]
    pub fn proc<P: Proc<'skip, R, Widget = Self>>(
        self,
        proc: impl Into<ProcArg<'skip, R, P>>,
    ) -> Self {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    #[inline]
    pub fn align<Align: crate::Align, Apply: crate::Apply>(mut self) -> Self {
        let res = Align::calc(self.renderer.get_parent(), &self.widget.size);
        Apply::apply(res, &mut self.widget.pos);
        self
    }

    #[inline]
    pub fn render<Style: crate::Style<R>>(mut self, style: impl Into<Style>) -> Self {
        style.into().render(&self.widget, &mut self.renderer);
        self
    }

    #[inline]
    pub fn on<On: crate::On<'skip, R, Self>>(
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
    pub fn child<W: Widget<'skip, R>>(
        mut self,
        f: impl FnOnce(W) -> W,
    ) -> Self {
        self.renderer
            .set_parent(&self.widget.size, &self.widget.pos);
        self.renderer = self.layout.layout(self.renderer, &self.widget.pos, f);
        self
    }

    #[inline]
    pub fn iter<Iter: Iterator, W: Widget<'skip, R>>(
        mut self,
        items: impl Into<IterArg<Iter>>,
        f: impl FnMut(W, Iter::Item) -> W,
    ) -> Self {
        self.renderer = self.layout.iter(self.renderer, &self.widget.pos, items, f);
        self
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

impl<'skip, TD: TextD<'skip, R>, R: Renderer> Text<'skip, TD, R> {
    #[inline]
    pub fn content(mut self, (text, font): (TD::Text, &'skip R::Font)) -> Self {
        self.content = Some((text, font));
        self
    }
    
    #[inline]
    pub fn render(mut self, color: impl Into<Color>) -> Self {
        if let Some((text, font)) = self.content {
            let t = TD::display(
                &mut self.renderer,
                &mut self.widget,
                text,
                font,
                color.into(),
            );
            self.content = Some((t, font));
        }
        self
    }


    #[inline]
    pub fn expr<T>(self, (expr_pack, f): (T, impl FnOnce(Self, T) -> Self)) -> Self {
        f(self, expr_pack)
    }

    #[inline]
    pub fn size(mut self, size: f32) -> Self {
        self.widget.size = size;
        if let Some((text, ref font)) = self.content {
            let t = TD::measure(&mut self.renderer, &mut self.widget, text, font);
            self.content = Some((t, font));
        } 
        self
    }

    #[inline]
    pub fn proc<P: Proc<'skip, R, Widget = Self>>(
        self,
        proc: impl Into<ProcArg<'skip, R, P>>,
    ) -> Self {
        let pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    pub fn align<Align: crate::Align, Apply: crate::Apply>(mut self) -> Self {
        let res = Align::calc(self.renderer.get_parent(), &self.widget.dim);
        Apply::apply(res, &mut self.widget.pos);
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.widget.pos, pos.into());
        self
    }

    pub fn on<On: crate::On<'skip, R, Self>>(
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

pub struct Time;

impl Operation for Time {
    type Item = Vec2<f32>;

    fn apply(initial_item: &mut Self::Item, diff: Self::Item) {
        initial_item.x *= diff.x;
        initial_item.y *= diff.y;
    }
}

pub trait Child<R: Renderer> {
    fn start(&self, renderer: R, dim: &Vec2<f32>, pos: &Vec2<f32>) -> R;
    fn end(&self, renderer: R) -> R;
}

pub struct Clip;

impl<R: Renderer> Child<R> for Clip {
    fn start(&self, mut renderer: R, dim: &Vec2<f32>, pos: &Vec2<f32>) -> R {
        renderer.start_clip(dim, pos);
        renderer
    }

    fn end(&self, mut renderer: R) -> R {
        renderer.end_clip();
        renderer
    }
}

pub struct Overflow;

impl<R: Renderer> Child<R> for Overflow {
    fn start(&self, renderer: R, _dim: &Vec2<f32>, _pos: &Vec2<f32>) -> R {
        renderer
    }

    fn end(&self, renderer: R) -> R {
        renderer
    }
}

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

    fn call(mut f: impl FnMut(Self::Arg) -> Self::FnOut, widget: W, cursor_pos: Vec2<f32>) -> W {
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

pub trait Style<R: Renderer> {
    fn render(self, div: &DivW, renderer: &mut R);
}

pub struct Plain<Color: Into<crate::Color> = ()> {
    pub color: Color,
    pub rad: f32,
}

pub struct Image<'skip, R: Renderer, Color: Into<crate::Color> = ()> { 
    pub src: &'skip R::Image,
    pub tint: Color,
}

impl<Color: Into<crate::Color>, R: Renderer> Style<R> for Plain<Color> {
    fn render(self, div: &DivW, renderer: &mut R) {
        renderer.render_div(div, self.color.into(), self.rad);
    }
}

impl<'skip, R: Renderer> Style<R> for Image<'skip, R> {
    fn render(self, div: &DivW, renderer: &mut R) {
        renderer.render_img(div, self.tint.into(), self.src);
        //        renderer.render_img(div, self.tint.into(), self.img_id);
    }
}

impl<Color: Into<crate::Color>> From<Color> for Plain<Color> {
    fn from(value: Color) -> Self {
        Self {
            color: value,
            rad: 0.0,
        }
    }
}

impl<Color: Into<crate::Color>> From<(Color, f32)> for Plain<Color> {
    fn from(value: (Color, f32)) -> Self {
        Self {
            color: value.0,
            rad: value.1,
        }
    }
}

impl<'skip, R: Renderer> From<&'skip R::Image> for Image<'skip, R> {
    fn from(value: &'skip R::Image) -> Self {
        Self {
            src: value,
            tint: ().into(),
        }
    }
}

impl<'skip, R: Renderer, Color: Into<crate::Color>> From<(&'skip R::Image, Color)>
    for Image<'skip, R, Color>
{
    fn from(value: (&'skip R::Image, Color)) -> Self {
        Self {
            src: value.0,
            tint: value.1,
        }
    }
}

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
    fn measure_paragraph(&mut self, text_w: &mut TextW, text: &str, paragraph: &mut Option<Self::Paragraph>, font: &Self::Font);
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

impl<'skip, R: Renderer, L: LayoutRule> Widget<'skip, R> for Div<L, R> {
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

impl<'skip, TD: TextD<'skip, R>, R: Renderer> Widget<'skip, R> for Text<'skip, TD, R>
where
    R::Font: 'skip,
{
    #[inline]
    fn inherit<PO: Into<Vec2<f32>>>(pos: PO, renderer: R) -> Self {
        Self {
            widget: TextW {
                dim: ().into(),
                pos: pos.into(),
                size: 10.0,
            },
            content: None,
            renderer,
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<'skip, R: Renderer, P: Proc<'skip, R>> From<(P, P::Arg)> for ProcArg<'skip, R, P> {
    fn from(value: (P, P::Arg)) -> Self {
        Self {
            proc: value.0,
            arg: value.1,
            ph: PhantomData::default(),
        }
    }
}

impl<'skip, R: Renderer, P: Proc<'skip, R, Arg = ()>> From<(P)> for ProcArg<'skip, R, P> {
    fn from(value: (P)) -> Self {
        Self {
            proc: value,
            arg: (),
            ph: PhantomData::default(),
        }
    }
}

struct IterArg<Iter: Iterator> {
    pub items: Iter,
    pub column: Option<usize>,
}

impl<Iter: Iterator> From<(Iter, usize)> for IterArg<Iter> {
    #[inline]
    fn from(value: (Iter, usize)) -> Self {
        Self {
            items: value.0,
            column: Some(value.1),
        }
    }
}

impl<Iter: Iterator> From<(Iter)> for IterArg<Iter> {
    fn from(value: (Iter)) -> Self {
        Self {
            items: value,
            column: None,
        }
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
