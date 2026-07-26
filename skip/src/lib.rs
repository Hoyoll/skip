pub mod cn;

mod builtin;

use std::marker::PhantomData;

use crate::builtin::{ProcArg, Widget};
#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct Circle<R: Renderer> {
    widget: CircleW,
    renderer: R
}

pub struct CircleW {
    pub radius: f32,
    pub pos: Vec2<f32>,
}

pub type ImageId = usize;

pub struct Text<'skip, R: Renderer> {
    widget: TextW<'skip>,
    renderer: R,
}

pub struct TextW<'skip> {
    pub text: &'skip str,
    pub font_id: Font,
    pub size: f32,
    pub pos: Vec2<f32>,
}

pub type Font = usize;

pub struct Div<R: Renderer> {
    widget: DivW,
    renderer: R,
}

pub struct DivW {
    pub size: Vec2<f32>,
    pub pos: Vec2<f32>,
}

pub struct Layout {
    pub offset: Vec2<f32>,
    pub pos: Vec2<f32>,
    pub size: Vec2<f32>,
    pub gap: f32,
}

pub struct Horizontal<R: Renderer> {
    layout: Layout,
    renderer: R,
}

pub struct Vertical<R: Renderer> {
    layout: Layout,
    renderer: R,
}

impl<'skip, R: Renderer> Horizontal<R> {
    #[inline]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.renderer.change_cursor(cursor);
        self
    }

    #[inline]
    pub fn new(renderer: R) -> Self {
        Self {
            layout: Layout {
                offset: ().into(),
                pos: ().into(),
                size: ().into(),
                gap: 0.0,
            },
            renderer,
        }
    }

    #[inline]
    pub fn add<W: Widget<'skip, R>>(
        mut self,
        mut f: impl FnOnce(W) -> W,
    ) -> Self {
        let mut w = f(W::inherit(
            (0.0, 0.0),
            (self.layout.pos.x + self.layout.offset.x, self.layout.pos.y),
            self.renderer,
        ));
        let size = w.size();
        self.layout.offset.x += size.x;
        self.renderer = w.renderer();
        self.layout.offset.x += self.layout.gap;
        self
    }

    #[inline]
    pub fn gap(mut self, gap: f32) -> Self {
        self.layout.gap = gap;
        self
    }

    #[inline]
    pub fn proc<P: Proc<'skip, R, Widget = Self>>(
        self,
        proc: impl Into<ProcArg<'skip, R, P>>,
    ) -> Self {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    pub fn on<On: crate::On<'skip, R, Self, Out = Self>>(mut self, f: impl FnMut(On::Arg<'_>) -> On::FnOut) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.layout.pos.x)
            && (mouse_pos.y >= self.layout.pos.y)
            && (mouse_pos.x <= (self.layout.pos.x + self.layout.size.x))
            && (mouse_pos.y <= (self.layout.pos.y + self.layout.size.y));
        if !hovered {
            return self;
        }
        On::call(f, self, mouse_pos) 
    }
    
    pub fn canvas_size(&mut self) -> Vec2<f32> {
        self.renderer.canvas_size()
    }


    #[inline]
    pub fn iter<Iter: Iterator, W: Widget<'skip, R>>(
        mut self,
        items: impl Into<IterArg<Iter>>,
        mut f: impl FnMut(W, Iter::Item) -> W,
    ) -> Self {
        let mut w;
        let mut iter_arg = items.into();
        match iter_arg.column {
            None => {
                for item in iter_arg.items.by_ref() {
                    w = f(
                        W::inherit(
                        (0.0, 0.0),
                        (self.layout.pos.x + self.layout.offset.x, self.layout.pos.y),
                        self.renderer,
                        ),
                    item,
                    );
                    let size = w.size();
                    self.layout.offset.x += size.x;
                    self.renderer = w.renderer();
                    self.layout.offset.x += self.layout.gap;
                }
            }
            Some(column) => {
                let mut limit: usize = 0;
                for item in iter_arg.items.by_ref() {                    
                    w = f(
                        W::inherit(
                        (0.0, 0.0),
                        (self.layout.pos.x + self.layout.offset.x, self.layout.pos.y + self.layout.offset.y),
                        self.renderer,
                        ),
                    item,
                    );
                    let size = w.size();
                    self.layout.offset.x += size.x;
                    self.renderer = w.renderer();
                    self.layout.offset.x += self.layout.gap;
                    limit += 1;
                    if column == limit {
                        limit = 0;
                        self.layout.offset.x = 0.0;
                        self.layout.offset.y += size.y + self.layout.gap;
                    }
                }
            }
        }
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.layout.pos,pos.into());
        self
    }

}


impl<'skip, R: Renderer> Vertical<R> {
    #[inline]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.renderer.change_cursor(cursor);
        self
    }

    #[inline]
    pub fn new(renderer: R) -> Self {
        Self {
            layout: Layout {
                offset: ().into(),
                size: ().into(),
                pos: ().into(),
                gap: 0.0,
            },
            renderer,
        }
    }

    #[inline]
    pub fn add<W: Widget<'skip, R>>(
        mut self,
        mut f: impl FnOnce(W) -> W,
    ) -> Self {
        let mut w = f(W::inherit(
            (0.0, 0.0),
            (self.layout.pos.x, self.layout.pos.y + self.layout.offset.y),
            self.renderer,
        ));
        let size = w.size();
        self.layout.offset.y += size.y;
        self.renderer = w.renderer();
        self.layout.offset.y += self.layout.gap;
        self
    }

    #[inline]
    pub fn gap(mut self, gap: f32) -> Self {
        self.layout.gap = gap;
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.layout.pos,pos.into());
        self
    }
    #[inline]
    pub fn proc<P: Proc<'skip, R, Widget = Self>>(
        self,
        proc: impl Into<ProcArg<'skip,R, P>>,
    ) -> Self {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }
    
    #[inline]
    pub fn iter<Iter: Iterator, W: Widget<'skip, R>>(
        mut self,
        mut items: impl Into<IterArg<Iter>>,
        mut f: impl FnMut(W, Iter::Item) -> W,
    ) -> Self {
        let mut w;
        let mut iter_arg = items.into();
        match iter_arg.column {
            None => {
                for item in iter_arg.items.by_ref() {
                    w = f(
                        W::inherit(
                        (0.0, 0.0),
                        (self.layout.pos.x, self.layout.pos.y + self.layout.offset.y),
                        self.renderer,
                        ),
                    item,
                    );
                    let size = w.size();
                    self.layout.offset.y += size.y;
                    self.renderer = w.renderer();
                    self.layout.offset.y += self.layout.gap;
                }
            }
            Some(column) => {
                let mut limit: usize = 0;
                for item in iter_arg.items.by_ref() {                    
                    w = f(
                        W::inherit(
                        (0.0, 0.0),
                        (self.layout.pos.x + self.layout.offset.x, self.layout.pos.y + self.layout.offset.y),
                        self.renderer,
                        ),
                    item,
                    );
                    let size = w.size();
                    self.layout.offset.y += size.y;
                    self.renderer = w.renderer();
                    self.layout.offset.y += self.layout.gap;
                    limit += 1;
                    if column == limit {
                        limit = 0;
                        self.layout.offset.y = 0.0;
                        self.layout.offset.x += size.x + self.layout.gap;
                    }
                }
            }
        }
        self
    }

    pub fn on<On: crate::On<'skip, R, Self, Out = Self>>(mut self, f: impl FnMut(On::Arg<'_>) -> On::FnOut) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.layout.pos.x)
            && (mouse_pos.y >= self.layout.pos.y)
            && (mouse_pos.x <= (self.layout.pos.x + self.layout.size.x))
            && (mouse_pos.y <= (self.layout.pos.y + self.layout.size.y));
        if !hovered {
            return self;
        }
        On::call(f, self, mouse_pos) 
    }

    pub fn canvas_size(&mut self) -> Vec2<f32> {
        self.renderer.canvas_size()
    }
}

impl<'skip, R: Renderer> Circle<R> {
    #[inline]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.renderer.change_cursor(cursor);
        self
    }

    #[inline]
    pub fn radius<V: Into<f32>>(mut self, rad: V) -> Self {
        self.widget.radius = rad.into();
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.widget.pos,pos.into());
        self
    }
    #[inline]
    pub fn render<C: Into<Color>>(mut self, color: C) -> Self {
        self.renderer.render_circle(&self.widget, color.into());
        self
    }
    #[inline]
    pub fn proc<PA: Into<ProcArg<'skip, R, P>>,P: Proc<'skip, R, Widget = Self>>(
        self,
        proc: PA,
    ) -> Self {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }
}

impl<'skip, R: Renderer> Div<R> {
    #[inline]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.renderer.change_cursor(cursor);
        self
    }
    
    #[inline]
    fn get_parent(&self) -> (Vec2<f32>, Vec2<f32>) {
        self.renderer.get_parent()
    }
   
    #[inline]
    pub fn proc<PA: Into<ProcArg<'skip, R, P>>,P: Proc<'skip, R, Widget = Self>>(
        self,
        proc: PA,
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
    pub fn render<Style: crate::Style>(mut self, style: impl Into<Style>) -> Self {
        style.into().render(&self.widget, &mut self.renderer);
        self
    }

    pub fn on<On: crate::On<'skip, R, Self, Out = Self>>(mut self, f: impl FnMut(On::Arg<'_>) -> On::FnOut) -> Self {
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
    pub fn child<W: Widget<'skip, R>, Child: crate::Child<R>>(
        mut self,
        f: impl FnOnce(W) -> W,
    ) -> Self {
        self.renderer.set_parent(&self.widget.size, &self.widget.pos);
        
        let w = f(W::inherit(
            &self.widget.size,
            &self.widget.pos,
            Child::start(self.renderer, &self.widget.size, &self.widget.pos),
        ));
        self.renderer = Child::end(w.renderer());
        self
    } 

    #[inline]
    pub fn size<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.widget.size,pos.into());
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.widget.pos,pos.into());
        self
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

pub trait Child<R: Renderer> {
    fn start(renderer: R, dim: &Vec2<f32>, pos: &Vec2<f32>) -> R;
    fn end(renderer: R) -> R;
}

pub struct Clip;

impl<R: Renderer> Child<R> for Clip {
    fn start(mut renderer: R, dim: &Vec2<f32>, pos: &Vec2<f32>) -> R {
        renderer.start_clip(dim, pos);
        renderer
    }

    fn end(mut renderer: R) -> R {
        renderer.end_clip();
        renderer
    }
}

pub struct Leak;

impl<R: Renderer> Child<R> for Leak {
    fn start(renderer: R, _dim: &Vec2<f32>, _pos: &Vec2<f32>) -> R {
        renderer
    }

    fn end(renderer: R) -> R {
        renderer
    }
}

pub trait On<'skip, R: Renderer, W: Widget<'skip, R>> {
    type Out;
    type FnOut;
    type Arg<'a>;
    fn call<F>(f: F, widget: W, cursor_pos: Vec2<f32>) -> Self::Out
    where
        F: for<'a> FnMut(Self::Arg<'a>) -> Self::FnOut;
    //fn call<F: FnMut(Self::Arg<'a>) -> Self::FnOut>(f: F, widget: W, cursor_pos: Vec2<f32>) -> Self::Out;
}

pub struct Hover;

impl<'skip, R: Renderer, W: Widget<'skip, R>> On<'skip, R, W> for Hover { 
    type Arg<'a> = (Vec2<f32>, W);

    type Out = W;
    type FnOut = W;

    fn call<F>(
        mut f: F,
        widget: W,
        cursor_pos: Vec2<f32>,
    ) -> Self::Out
    where
        F: for<'a> FnMut(Self::Arg<'a>) -> Self::FnOut,
    {
        f((cursor_pos, widget))
    } 
}

pub struct Keys;

impl<'skip, R: Renderer, W: Widget<'skip, R>> On<'skip, R, W> for Keys {
    type Out = W;
    //type Arg = &(Mouse, State);
    type FnOut = ();
    
    type Arg<'a> = &'a (Mouse, State);

    fn call<F>(mut f: F, widget: W, _cursor_pos: Vec2<f32>) -> Self::Out
    where
        F: for<'a> FnMut(Self::Arg<'a>) -> Self::FnOut,
    {
        widget.iter_mouse(f);
        widget
    }
} 

pub trait Style {
    fn render<R: Renderer>(self, div: &DivW, renderer: &mut R);
}

pub struct Plain<Color: Into<crate::Color> = ()> {
    pub color: Color,
    pub rad: f32,
}

pub struct Image<Color: Into<crate::Color> = ()> {
    pub img_id: ImageId,
    pub tint: Color
}

impl<Color: Into<crate::Color>> Style for Plain<Color> {
    fn render<R: Renderer>(self, div: &DivW, renderer: &mut R) {
        renderer.render_div(div, self.color.into(), self.rad);
    }
}

impl Style for Image  {
    fn render<R: Renderer>(self, div: &DivW, renderer: &mut R) {
        renderer.render_img(div, self.tint.into(), self.img_id);
    }
}

impl<Color: Into<crate::Color>> From<Color> for Plain<Color> {
    fn from(value: Color) -> Self {
        Self { color: value, rad: 0.0 }
    }
}

impl<Color: Into<crate::Color>> From<(Color, f32)> for Plain<Color> {
    fn from(value: (Color, f32)) -> Self {
        Self { color: value.0, rad: value.1 }
    }
}

impl From<ImageId> for Image<()> {
    fn from(value: ImageId) -> Self {
        Self { img_id: value, tint: () }
    }
}

impl<Color: Into<crate::Color>> From<(ImageId, Color)> for Image<Color> {
    fn from(value: (ImageId, Color)) -> Self {
        Self { img_id: value.0, tint: value.1 }
    }
}

impl<'skip, R: Renderer> Text<'skip, R> {
    #[inline]
    fn get_parent(&self) -> (Vec2<f32>, Vec2<f32>) {
        self.renderer.get_parent()
    }

    #[inline]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.renderer.change_cursor(cursor);
        self
    }

    #[inline]
    pub fn size(mut self, size: f32) -> Self {
        self.widget.size = size;
        self
    }

    fn get_size(&mut self) -> Vec2<f32> {
        self.size()
    }

    #[inline]
    pub fn text(mut self, text: &'skip str) -> Self {
        self.widget.text = text;
        self
    }

    #[inline]
    pub fn font_id(mut self, font_id: usize) -> Self {
        self.widget.font_id = font_id;
        self
    }

    #[inline]
    pub fn render<C: Into<Color>>(mut self, color: C) -> Self {
        self.renderer.render_text(&self.widget, color.into());
        self
    }
    #[inline]
    pub fn proc<P: Proc<'skip, R, Widget = Self>>(
        self,
        proc: impl Into<ProcArg<'skip, R, P>>,
    ) -> Self {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    pub fn align<Align: crate::Align, Apply: crate::Apply>(mut self) -> Self {
        let res = Align::calc(self.get_parent(), self.get_size());
        Apply::apply(res, &mut self.widget.pos);
        self
    }

    #[inline]
    pub fn position<Op: Operation<Item = Vec2<f32>>>(mut self, pos: impl Into<Op::Item>) -> Self {
        Op::apply(&mut self.widget.pos,pos.into());
        self
    }
}

pub trait Align {
    fn calc(parent: impl Into<(Vec2<f32>, Vec2<f32>)>, child_dim: impl Into<Vec2<f32>>) -> Vec2<f32>;
}

pub struct Center;

impl Align for Center {
    fn calc(parent: impl Into<(Vec2<f32>, Vec2<f32>)>, child_dim: impl Into<Vec2<f32>>) -> Vec2<f32> {
        let (p_size, p_pos) = parent.into();
        let size = child_dim.into();
        let center_pos: Vec2<_> = (p_pos.x + p_size.x / 2.0, p_pos.y + p_size.y / 2.0).into();
        (center_pos.x - size.x / 2.0, center_pos.y - size.y / 2.0).into()
    }
}

pub struct End;

impl Align for End {
    fn calc(parent: impl Into<(Vec2<f32>, Vec2<f32>)>, child_dim: impl Into<Vec2<f32>>) -> Vec2<f32> {
        let (p_size, p_pos) = parent.into();
        let size = child_dim.into();
        let end_pos: Vec2<_> = (p_pos.x + p_size.x, p_pos.y + p_size.y).into();
        (end_pos.x - size.x, end_pos.y - size.y).into()
    }
}

pub struct Start;

impl Align for Start {
    fn calc(parent: impl Into<(Vec2<f32>, Vec2<f32>)>, _child_dim: impl Into<Vec2<f32>>) -> Vec2<f32> {
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
    fn render_text<'skip>(&mut self, text: &TextW<'skip>, color: Color);
    fn render_div(&mut self, div: &DivW, color: Color, radius: f32);
    fn render_img(&mut self, img: &DivW, color: Color, image_id: ImageId);
    fn render_circle(&mut self, circle: &CircleW, color: Color);
    fn text_size<'skip>(&mut self, text: &TextW<'skip>) -> Vec2<f32>;
    fn start_clip(&mut self, dim: &Vec2<f32>, pos: &Vec2<f32>);
    fn mouse_pos(&mut self) -> Vec2<f32>;
    //fn mouse_state(&mut self) -> &Vec<(Mouse, State)>;
    fn end_clip(&mut self);
    fn canvas_size(&mut self) -> Vec2<f32>;
    fn change_cursor(&mut self, cursor: Cursor);
    fn set_parent<Dim: Into<Vec2<f32>>, Pos: Into<Vec2<f32>>>(&mut self, dim: Dim, pos: Pos);
    fn get_parent(&self) -> (Vec2<f32>, Vec2<f32>);
    fn iter_mouse<F: FnMut(&(Mouse, State))>(&self, f: F);
}

pub trait Proc<'skip, R: Renderer> {
    type Widget: Widget<'skip, R>;
    type Arg;
    fn consume(self, widget: Self::Widget, argv: Self::Arg) -> Self::Widget;
}

pub enum Cursor {
    Default,
    Pointer,
}

impl<'skip, R: Renderer> Widget<'skip, R> for Circle<R> {
    fn inherit<P: Into<Vec2<f32>>, PO: Into<Vec2<f32>>>(dim: P, pos: PO, renderer: R) -> Self {
        Self { widget: CircleW { 
            radius: dim.into().x / 2.0, 
            pos: pos.into() 
        }, renderer }
    }
    fn renderer(self) -> R {
        self.renderer
    }
    fn size(&mut self) -> Vec2<f32> {
        (self.widget.radius, self.widget.radius).into()
    }

    fn iter_mouse<F: FnMut(&(Mouse, State))>(&self, f: F) {
        self.renderer.iter_mouse(f);
    }
}

impl<'skip, R: Renderer> Widget<'skip, R> for Horizontal<R> {
    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }

    #[inline]
    fn inherit<P: Into<Vec2<f32>>, PO: Into<Vec2<f32>>>(dim: P, pos: PO, renderer: R) -> Self {
        //let p = pos.into();
        Self {
            layout: Layout {
                offset: ().into(),
                pos: pos.into(),
                size: dim.into(),
                gap: 0.0,
            },
            renderer,
        }
    }

    #[inline]
    fn size(&mut self) -> Vec2<f32> {
        (&self.layout.size).into()
    }

    fn iter_mouse<F: FnMut(& (Mouse, State))>(&self, f: F) {
        self.renderer.iter_mouse(f);
    }

}

impl<'skip, R: Renderer> Widget<'skip, R> for Vertical<R> {
    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }
    #[inline]
    fn inherit<P: Into<Vec2<f32>>, PO: Into<Vec2<f32>>>(dim: P, pos: PO, renderer: R) -> Self {
        //let p = pos.into();
        Self {
            layout: Layout {
                offset: ().into(),
                pos: pos.into(),
                size: dim.into(),
                gap: 0.0,
            },
            renderer,
        }
    }
    #[inline]
    fn size(&mut self) -> Vec2<f32> {
        (&self.layout.size).into()
    }

    fn iter_mouse<F: FnMut(& (Mouse, State))>(&self, f: F) {
        self.renderer.iter_mouse(f);
    } 
}

impl<'skip, R: Renderer> Widget<'skip, R> for Div<R> {
    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }
    #[inline]
    fn inherit<P: Into<Vec2<f32>>, PO: Into<Vec2<f32>>>(dim: P, pos: PO, renderer: R) -> Self {
        let widget: DivW = DivW { size: dim.into(),pos: pos.into() };
        Self { widget, renderer }
    }
    #[inline]
    fn size(&mut self) -> Vec2<f32> {
        (self.widget.size.x, self.widget.size.y).into()
    }
    fn iter_mouse<F: FnMut(& (Mouse, State))>(&self, f: F) {
        self.renderer.iter_mouse(f);
    } 
}

impl<'skip, R: Renderer> Widget<'skip, R> for Text<'skip, R> {
    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }
    #[inline]
    fn inherit<P: Into<Vec2<f32>>, PO: Into<Vec2<f32>>>(_dim: P, pos: PO, renderer: R) -> Self {
        let mut widget: TextW<'_> = ().into();
        widget.pos = pos.into();
        Self { widget, renderer }
    }
    
    #[inline]
    fn size(&mut self) -> Vec2<f32> {
        self.renderer.text_size(&self.widget)
    }

    fn iter_mouse<F: FnMut(& (Mouse, State))>(&self, f: F) {
        self.renderer.iter_mouse(f);
    } 
}

pub enum State {
    Pressed,
    Released
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
    Unknown
}


#[derive(Debug)]
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

impl< 'skip, R: Renderer, P: Proc<'skip, R>> From<(P, P::Arg)> for ProcArg<'skip, R, P>
{
    fn from(value: (P, P::Arg)) -> Self {
        Self { proc: value.0, arg: value.1, ph: PhantomData::default() }
    }
}

impl<'skip, R: Renderer, P: Proc<'skip, R, Arg = ()>> From<(P)> for ProcArg<'skip, R, P> 
{
    fn from(value: (P)) -> Self {
        Self { proc: value, arg: (), ph: PhantomData::default() }
    }
}

struct IterArg<Iter: Iterator> {
    pub items: Iter,
    pub column: Option<usize>
}

impl<'skip> From<()> for TextW<'skip> {
    #[inline]
    fn from(_value: ()) -> Self {
        Self {
            text: "",
            font_id: 0,
            size: 0.0, 
            pos: ().into(),
        }
    }
}


impl<Iter: Iterator> From<(Iter, usize)> for IterArg<Iter> {
    #[inline]
    fn from(value: (Iter, usize)) -> Self {
        Self { items: value.0, column: Some(value.1) }
    }
}

impl<Iter: Iterator> From<(Iter)> for IterArg<Iter> {
    fn from(value: (Iter)) -> Self {
        Self { items: value, column: None }
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
        Self { r: value.r, g: value.g, b: value.b, a: value.a }
    }
}


impl From<&mut Color> for Color {
    fn from(value: &mut Color) -> Self {
        Self { r: value.r, g: value.g, b: value.b, a: value.a }
    }
}
