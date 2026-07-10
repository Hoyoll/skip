use std::marker::PhantomData;

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct Image<R: Renderer> {
    widget: ImageW,
    renderer: R,
}

pub type ImageId = usize;

pub struct ImageW {
    pub image_id: ImageId,
    pub pos: Vec2<f32>,
    pub size: Vec2<f32>,
    pub tint: Color,
}

pub struct Text<'skip, R: Renderer> {
    widget: TextW<'skip>,
    renderer: R,
}

pub struct TextW<'skip> {
    pub text: &'skip str,
    pub font_id: Font,
    pub color: Color,
    pub pos: Vec2<f32>,
}

pub type Font = usize;

pub struct Div<R: Renderer> {
    widget: DivW,
    renderer: R,
}

pub struct DivW {
    pub size: Vec2<f32>,
    pub rad: f32,
    pub color: Color,
    pub pos: Vec2<f32>,
}

pub struct Layout {
    pub offset: f32,
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
    pub fn new(renderer: R) -> Self {
        Self {
            layout: Layout {
                offset: 0.0,
                pos: ().into(),
                size: ().into(),
                gap: 0.0,
            },
            renderer,
        }
    }

    #[inline]
    pub fn add<W: Widget<'skip, R>, F: FnMut(W) -> WO, WO: Widget<'skip, R>>(
        mut self,
        mut f: F,
    ) -> Self {
        self.layout.offset += self.layout.gap;
        let mut w = f(W::inherit(
            (0.0, 0.0),
            (self.layout.pos.x + self.layout.offset, self.layout.pos.y),
            self.renderer,
        ));
        let size = w.size();
        self.layout.offset += size.x;
        self.renderer = w.renderer();
        self
    }

    #[inline]
    pub fn gap(mut self, gap: f32) -> Self {
        self.layout.gap = gap;
        self
    }

    #[inline]
    pub fn padding<V: Into<Vec2<f32>>>(mut self, v: V) -> Self {
        let pad = v.into();
        self.layout.pos.x += pad.x;
        self.layout.pos.y += pad.y;
        self
    }

    #[inline]
    pub fn proc<
        PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>,
        P: Proc<'skip, Self, Out, R, Arg>,
        Out: Widget<'skip, R>,
        Arg,
    >(
        self,
        proc: PA,
    ) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    #[inline]
    pub fn iter<Iter: Iterator, F: FnMut(W, Iter::Item) -> W, W: Widget<'skip, R>>(
        mut self,
        mut items: Iter,
        mut f: F,
    ) -> Self {
        let mut w;
        for item in items.by_ref() {
            self.layout.offset += self.layout.gap;
            w = f(
                W::inherit(
                    (0.0, 0.0),
                    (self.layout.pos.x + self.layout.offset, self.layout.pos.y),
                    self.renderer,
                ),
                item,
            );
            let size = w.size();
            self.layout.offset += size.x;
            self.renderer = w.renderer();
        }
        self
    }

    pub fn on<F: FnMut(&On)>(mut self, mut f: F) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let mouse = self.renderer.mouse_state();
        let hovered = (mouse_pos.x >= self.layout.pos.x)
            && (mouse_pos.y >= self.layout.pos.y)
            && (mouse_pos.x <= (self.layout.pos.x + self.layout.size.x))
            && (mouse_pos.y <= (self.layout.pos.y + self.layout.size.y));
        if !hovered {
            return self;
        }
        for on in mouse {
            f(on);
        }
        self
    }

    #[inline]
    pub fn key<F: FnMut(&Key)>(mut self, mut f: F) -> Self {
        let keys = self.renderer.key_state();
        for on in keys {
            f(on);
        }
        self
    }

    pub fn hover<F: FnMut(Vec2<f32>, Self) -> Self>(mut self, mut f: F) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.layout.pos.x)
            && (mouse_pos.y >= self.layout.pos.y)
            && (mouse_pos.x <= (self.layout.pos.x + self.layout.size.x))
            && (mouse_pos.y <= (self.layout.pos.y + self.layout.size.y));
        if hovered {
            return f(mouse_pos, self);
        }
        self
    }
}

impl<'skip, R: Renderer> Vertical<R> {
    #[inline]
    pub fn new(renderer: R) -> Self {
        Self {
            layout: Layout {
                offset: 0.0,
                size: ().into(),
                pos: ().into(),
                gap: 0.0,
            },
            renderer,
        }
    }

    #[inline]
    pub fn add<W: Widget<'skip, R>, F: FnMut(W) -> WO, WO: Widget<'skip, R>>(
        mut self,
        mut f: F,
    ) -> Self {
        self.layout.offset += self.layout.gap;
        let mut w = f(W::inherit(
            (0.0, 0.0),
            (self.layout.pos.x, self.layout.pos.y + self.layout.offset),
            self.renderer,
        ));
        let size = w.size();
        self.layout.offset += size.y;
        self.renderer = w.renderer();
        self
    }

    #[inline]
    pub fn gap(mut self, gap: f32) -> Self {
        self.layout.gap = gap;
        self
    }

    #[inline]
    pub fn padding<V: Into<Vec2<f32>>>(mut self, v: V) -> Self {
        let pad = v.into();
        self.layout.pos.x += pad.x;
        self.layout.pos.y += pad.y;
        self
    }

    #[inline]
    pub fn proc<
        PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>,
        P: Proc<'skip, Self, Out, R, Arg>,
        Out: Widget<'skip, R>,
        Arg,
    >(
        self,
        proc: PA,
    ) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    #[inline]
    pub fn iter<Iter: Iterator, F: FnMut(W, Iter::Item) -> W, W: Widget<'skip, R>>(
        mut self,
        mut items: Iter,
        mut f: F,
    ) -> Self {
        let mut w;
        for item in items.by_ref() {
            self.layout.offset += self.layout.gap;
            w = f(
                W::inherit(
                    (0.0, 0.0),
                    (self.layout.pos.x, self.layout.pos.y + self.layout.offset),
                    self.renderer,
                ),
                item,
            );
            let size = w.size();
            self.layout.offset += size.y;
            self.renderer = w.renderer();
        }
        self
    }

    pub fn on<F: FnMut(&On)>(mut self, mut f: F) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let mouse = self.renderer.mouse_state();
        let hovered = (mouse_pos.x >= self.layout.pos.x)
            && (mouse_pos.y >= self.layout.pos.y)
            && (mouse_pos.x <= (self.layout.pos.x + self.layout.size.x))
            && (mouse_pos.y <= (self.layout.pos.y + self.layout.size.y));
        if !hovered {
            return self;
        }
        for on in mouse {
            f(on);
        }
        self
    }

    #[inline]
    pub fn key<F: FnMut(&Key)>(mut self, mut f: F) -> Self {
        let keys = self.renderer.key_state();
        for on in keys {
            f(on);
        }
        self
    }

    pub fn hover<F: FnMut(Vec2<f32>, Self) -> Self>(mut self, mut f: F) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.layout.pos.x)
            && (mouse_pos.y >= self.layout.pos.y)
            && (mouse_pos.x <= (self.layout.pos.x + self.layout.size.x))
            && (mouse_pos.y <= (self.layout.pos.y + self.layout.size.y));
        if hovered {
            return f(mouse_pos, self);
        }
        self
    }
}

impl<'skip, R: Renderer> Image<R> {
    #[inline]
    pub fn child<W: Widget<'skip, R>, F: FnMut(W) -> WO, WO: Widget<'skip, R>>(
        mut self,
        mut f: F,
    ) -> Self {
        let w = f(W::inherit(
            &self.widget.size,
            &self.widget.pos,
            self.renderer,
        ));
        self.renderer = w.renderer();
        self
    }

    #[inline]
    pub fn image_id(mut self, img: ImageId) -> Self {
        self.widget.image_id = img;
        self
    }

    #[inline]
    pub fn size<V: Into<Vec2<f32>>>(mut self, dim: V) -> Self {
        self.widget.size = dim.into();
        self
    }

    #[inline]
    pub fn pos<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }

    #[inline]
    pub fn tint<C: Into<Color>>(mut self, color: C) -> Self {
        self.widget.tint = color.into();
        self
    }

    #[inline]
    pub fn proc<
        PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>,
        P: Proc<'skip, Self, Out, R, Arg>,
        Out: Widget<'skip, R>,
        Arg,
    >(
        self,
        proc: PA,
    ) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }
    #[inline]
    pub fn horizontal<F: FnMut(Horizontal<R>) -> W, W: Widget<'skip, R>>(
        mut self,
        mut f: F,
    ) -> Self {
        let w = f(Horizontal {
            layout: Layout {
                pos: (&self.widget.pos).into(),
                offset: 0.0,
                size: (&self.widget.size).into(),
                gap: 0.0,
            },
            renderer: self.renderer,
        });
        self.renderer = w.renderer();
        self
    }

    #[inline]
    pub fn vertical<F: FnMut(Vertical<R>) -> W, W: Widget<'skip, R>>(mut self, mut f: F) -> Self {
        let w = f(Vertical {
            layout: Layout {
                pos: (&self.widget.pos).into(),
                offset: 0.0,
                size: (&self.widget.size).into(),
                gap: 0.0,
            },
            renderer: self.renderer,
        });
        self.renderer = w.renderer();
        self
    }

    pub fn hover<F: FnMut(Vec2<f32>, Self) -> Self>(mut self, mut f: F) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.widget.pos.x)
            && (mouse_pos.y >= self.widget.pos.y)
            && (mouse_pos.x <= (self.widget.pos.x + self.widget.size.x))
            && (mouse_pos.y <= (self.widget.pos.y + self.widget.size.y));
        if hovered {
            return f(mouse_pos, self);
        }
        self
    }

    pub fn on<F: FnMut(&On)>(mut self, mut f: F) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let mouse = self.renderer.mouse_state();
        let hovered = (mouse_pos.x >= self.widget.pos.x)
            && (mouse_pos.y >= self.widget.pos.y)
            && (mouse_pos.x <= (self.widget.pos.x + self.widget.size.x))
            && (mouse_pos.y <= (self.widget.pos.y + self.widget.size.y));
        if !hovered {
            return self;
        }
        for on in mouse {
            f(on);
        }
        self
    }

    #[inline]
    pub fn key<F: FnMut(&Key)>(mut self, mut f: F) -> Self {
        let keys = self.renderer.key_state();
        for on in keys {
            f(on);
        }
        self
    }

    pub fn padding<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        let pos = pos.into();
        self.widget.pos.x += pos.x;
        self.widget.pos.y += pos.y;
        self
    }
}

impl<'skip, R: Renderer> Div<R> {
    #[inline]
    pub fn child<W: Widget<'skip, R>, F: FnMut(W) -> WO, WO: Widget<'skip, R>>(
        mut self,
        mut f: F,
    ) -> Self {
        let w = f(W::inherit(
            &self.widget.size,
            &self.widget.pos,
            self.renderer,
        ));
        self.renderer = w.renderer();
        self
    }

    pub fn padding<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        let pos = pos.into();
        self.widget.pos.x += pos.x;
        self.widget.pos.y += pos.y;
        self
    }

    #[inline]
    pub fn pos<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }

    #[inline]
    pub fn size<V: Into<Vec2<f32>>>(mut self, dim: V) -> Self {
        self.widget.size = dim.into();
        self
    }

    #[inline]
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.widget.color = color.into();
        self
    }

    #[inline]
    pub fn render(mut self) -> Self {
        self.renderer.render_div(&self.widget);
        self
    }

    pub fn rad(mut self, rad: f32) -> Self {
        self.widget.rad = rad;
        self
    }

    pub fn hover<F: FnMut(Vec2<f32>, Self) -> Self>(mut self, mut f: F) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let hovered = (mouse_pos.x >= self.widget.pos.x)
            && (mouse_pos.y >= self.widget.pos.y)
            && (mouse_pos.x <= (self.widget.pos.x + self.widget.size.x))
            && (mouse_pos.y <= (self.widget.pos.y + self.widget.size.y));
        if hovered {
            return f(mouse_pos, self);
        }
        self
    }

    pub fn on<F: FnMut(&On)>(mut self, mut f: F) -> Self {
        let mouse_pos = self.renderer.mouse_pos();
        let mouse = self.renderer.mouse_state();
        let hovered = (mouse_pos.x >= self.widget.pos.x)
            && (mouse_pos.y >= self.widget.pos.y)
            && (mouse_pos.x <= (self.widget.pos.x + self.widget.size.x))
            && (mouse_pos.y <= (self.widget.pos.y + self.widget.size.y));
        if !hovered {
            return self;
        }
        for on in mouse {
            f(on);
        }
        self
    }

    #[inline]
    pub fn key<F: FnMut(&Key)>(mut self, mut f: F) -> Self {
        let keys = self.renderer.key_state();
        for on in keys {
            f(on);
        }
        self
    }

    #[inline]
    pub fn proc<
        PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>,
        P: Proc<'skip, Self, Out, R, Arg>,
        Out: Widget<'skip, R>,
        Arg,
    >(
        self,
        proc: PA,
    ) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    #[inline]
    pub fn horizontal<F: FnMut(Horizontal<R>) -> W, W: Widget<'skip, R>>(
        mut self,
        mut f: F,
    ) -> Self {
        let w = f(Horizontal {
            layout: Layout {
                pos: (&self.widget.pos).into(),
                offset: 0.0,
                size: (&self.widget.size).into(),
                gap: 0.0,
            },
            renderer: self.renderer,
        });
        self.renderer = w.renderer();
        self
    }

    #[inline]
    pub fn vertical<F: FnMut(Vertical<R>) -> W, W: Widget<'skip, R>>(mut self, mut f: F) -> Self {
        let w = f(Vertical {
            layout: Layout {
                pos: (&self.widget.pos).into(),
                offset: 0.0,
                size: (&self.widget.size).into(),
                gap: 0.0,
            },
            renderer: self.renderer,
        });
        self.renderer = w.renderer();
        self
    }

    #[inline]
    pub fn clip<W: Widget<'skip, R>, F: FnMut(W) -> WO, WO: Widget<'skip, R>>(
        mut self,
        mut f: F,
    ) -> Self {
        self.renderer.start_clip(&self.widget);
        let w = f(W::inherit(
            &self.widget.size,
            &self.widget.pos,
            self.renderer,
        ));
        self.renderer = w.renderer();
        self.renderer.end_clip();
        self
    }
}

impl<'skip, R: Renderer> Text<'skip, R> {
    #[inline]
    pub fn pos<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }

    pub fn padding<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        let pos = pos.into();
        self.widget.pos.x += pos.x;
        self.widget.pos.y += pos.y;
        self
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
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.widget.color = color.into();
        self
    }

    #[inline]
    pub fn render(mut self) -> Self {
        self.renderer.render_text(&self.widget);
        self
    }

    #[inline]
    pub fn proc<
        PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>,
        P: Proc<'skip, Self, Out, R, Arg>,
        Out: Widget<'skip, R>,
        Arg,
    >(
        self,
        proc: PA,
    ) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }
}

pub trait Renderer {
    fn render_text<'skip>(&mut self, text: &TextW<'skip>);
    fn render_div(&mut self, div: &DivW);
    fn render_img(&mut self, img: &ImageW);
    fn text_size<'skip>(&mut self, text: &TextW<'skip>) -> Vec2<f32>;
    fn start_clip(&mut self, dim: &DivW);
    fn mouse_pos(&mut self) -> Vec2<f32>;
    fn mouse_state(&mut self) -> &Vec<On>;
    fn key_state(&mut self) -> &Vec<Key>;
    fn end_clip(&mut self);
}

pub trait Proc<'skip, In: Widget<'skip, R>, Out: Widget<'skip, R>, R: Renderer, Arg> {
    fn consume(&mut self, widget: In, argv: Arg) -> Out;
}

pub(crate) struct ProcArg<
    'skip,
    P: Proc<'skip, In, Out, R, Arg>,
    In: Widget<'skip, R>,
    Out: Widget<'skip, R>,
    R: Renderer,
    Arg,
> {
    proc: P,
    arg: Arg,
    ph: PhantomData<(&'skip (), In, Out, Arg, R)>,
}

pub(crate) trait Widget<'skip, R: Renderer> {
    fn inherit<P: Into<Vec2<f32>>>(dim: P, pos: P, renderer: R) -> Self;
    fn renderer(self) -> R;
    fn size(&mut self) -> Vec2<f32>;
}

impl<'skip, R: Renderer> Widget<'skip, R> for Image<R> {
    #[inline]
    fn inherit<P: Into<Vec2<f32>>>(dim: P, pos: P, renderer: R) -> Self {
        Self {
            widget: ImageW {
                image_id: 0,
                pos: pos.into(),
                size: dim.into(),
                tint: ().into(),
            },
            renderer,
        }
    }

    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }

    #[inline]
    fn size(&mut self) -> Vec2<f32> {
        (&self.widget.size).into()
    }
}

impl<'skip, R: Renderer> Widget<'skip, R> for Horizontal<R> {
    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }

    #[inline]
    fn inherit<P: Into<Vec2<f32>>>(dim: P, pos: P, renderer: R) -> Self {
        //let p = pos.into();
        Self {
            layout: Layout {
                offset: 0.0,
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
}

impl<'skip, R: Renderer> Widget<'skip, R> for Vertical<R> {
    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }
    #[inline]
    fn inherit<P: Into<Vec2<f32>>>(dim: P, pos: P, renderer: R) -> Self {
        //let p = pos.into();
        Self {
            layout: Layout {
                offset: 0.0,
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
}

impl<'skip, R: Renderer> Widget<'skip, R> for Div<R> {
    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }
    #[inline]
    fn inherit<P: Into<Vec2<f32>>>(dim: P, pos: P, renderer: R) -> Self {
        let mut widget: DivW = ().into();
        widget.pos = pos.into();
        widget.size = dim.into();
        Self { widget, renderer }
    }
    #[inline]
    fn size(&mut self) -> Vec2<f32> {
        (self.widget.size.x, self.widget.size.y).into()
    }
}

impl<'skip, R: Renderer> Widget<'skip, R> for Text<'skip, R> {
    #[inline]
    fn renderer(self) -> R {
        self.renderer
    }
    #[inline]
    fn inherit<P: Into<Vec2<f32>>>(_dim: P, pos: P, renderer: R) -> Self {
        let mut widget: TextW<'_> = ().into();
        widget.pos = pos.into();
        Self { widget, renderer }
    }
    #[inline]
    fn size(&mut self) -> Vec2<f32> {
        self.renderer.text_size(&self.widget)
    }
}

pub enum On {
    Press(Mouse),
    Release(Mouse),
}

pub enum Mouse {
    Left,
    Right,
    Middle,
    Unknown,
}

pub enum Key {
    Press(&'static str),
    Release(&'static str),
}

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

impl<
    'skip,
    P: Proc<'skip, In, Out, R, Arg>,
    In: Widget<'skip, R>,
    Out: Widget<'skip, R>,
    R: Renderer,
    Arg,
> From<(P, Arg)> for ProcArg<'skip, P, In, Out, R, Arg>
{
    #[inline]
    fn from(value: (P, Arg)) -> Self {
        Self {
            proc: value.0,
            arg: value.1,
            ph: PhantomData::default(),
        }
    }
}

impl<
    'skip,
    P: Proc<'skip, In, Out, R, ()>,
    In: Widget<'skip, R>,
    Out: Widget<'skip, R>,
    R: Renderer,
> From<(P)> for ProcArg<'skip, P, In, Out, R, ()>
{
    #[inline]
    fn from(value: (P)) -> Self {
        Self {
            proc: value,
            arg: (),
            ph: PhantomData::default(),
        }
    }
}

impl From<(ImageId)> for ImageW {
    #[inline]
    fn from(value: (ImageId)) -> Self {
        Self {
            image_id: value,
            pos: ().into(),
            size: ().into(),
            tint: ().into(),
        }
    }
}

impl<C: Into<Color>> From<(ImageId, C)> for ImageW {
    #[inline]
    fn from(value: (ImageId, C)) -> Self {
        Self {
            image_id: value.0,
            pos: ().into(),
            size: ().into(),
            tint: value.1.into(),
        }
    }
}

impl<'skip> From<()> for TextW<'skip> {
    #[inline]
    fn from(_value: ()) -> Self {
        Self {
            text: "",
            font_id: 0,
            color: ().into(),
            pos: ().into(),
        }
    }
}

impl<'skip> From<(&'skip str)> for TextW<'skip> {
    #[inline]
    fn from(value: (&'skip str)) -> Self {
        Self {
            text: value.into(),
            font_id: 0,
            color: ().into(),
            pos: ().into(),
        }
    }
}

impl<'skip, Pos: Into<Vec2<f32>>> From<(&'skip str, Pos)> for TextW<'skip> {
    #[inline]
    fn from(value: (&'skip str, Pos)) -> Self {
        Self {
            text: value.0.into(),
            font_id: 0,
            color: ().into(),
            pos: value.1.into(),
        }
    }
}

impl<'skip> From<(&'skip str, Font)> for TextW<'skip> {
    #[inline]
    fn from(value: (&'skip str, Font)) -> Self {
        Self {
            text: value.0,
            font_id: value.1,
            color: ().into(),
            pos: ().into(),
        }
    }
}

impl<'skip, Col: Into<Color>, Pos: Into<Vec2<f32>>> From<(&'skip str, Font, Col, Pos)>
    for TextW<'skip>
{
    #[inline]
    fn from(value: (&'skip str, Font, Col, Pos)) -> Self {
        Self {
            text: value.0,
            font_id: value.1,
            color: value.2.into(),
            pos: value.3.into(),
        }
    }
}

impl From<()> for DivW {
    #[inline]
    fn from(_value: ()) -> Self {
        Self {
            size: ().into(),
            rad: 0.0,
            color: ().into(),
            pos: ().into(),
        }
    }
}

impl<Dim: Into<Vec2<f32>>, Pos: Into<Vec2<f32>>> From<(Dim, Pos)> for DivW {
    #[inline]
    fn from(value: (Dim, Pos)) -> Self {
        Self {
            size: value.0.into(),
            rad: 0.0,
            color: ().into(),
            pos: value.1.into(),
        }
    }
}

impl<Dim: Into<Vec2<f32>>, Pos: Into<Vec2<f32>>, Col: Into<Color>> From<(Dim, Pos, f32, Col)>
    for DivW
{
    #[inline]
    fn from(value: (Dim, Pos, f32, Col)) -> Self {
        Self {
            size: value.0.into(),
            rad: value.2,
            color: value.3.into(),
            pos: value.1.into(),
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
