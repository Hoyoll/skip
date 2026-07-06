use std::marker::PhantomData;

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct Image<R: Renderer> {
    widget: ImageW,
    renderer: R
}

pub type ImageId = usize;

pub struct ImageW {
    pub image_id: ImageId,
    pub pos: Vec2<f32>,
    pub dim: Vec2<f32>,
    pub tint: Color,
}

pub struct Text<'skip, R: Renderer> {
    widget: TextW<'skip>,
    renderer: R,
}

pub struct TextW<'skip> {
    pub text: &'skip str,
    pub font_id: Font,
    //pub size: usize,
    pub color: Color,
    pub pos: Vec2<f32>,
}

pub type Font = usize;

pub struct Div<R> {
    widget: DivW,
    renderer: R,
}

pub struct DivW {
    pub dim: Vec2<f32>,
    pub rad: f32,
    pub color: Color,
    pub pos: Vec2<f32>, 
}

pub struct Horizontal<R: Renderer> {
    y_anchor: f32,
    x_offset: f32,
    dim: Vec2<f32>,
    gap: f32,
    renderer: R,
}

pub struct Vertical<R: Renderer> {
    x_anchor: f32,
    y_offset: f32,
    dim: Vec2<f32>,
    gap: f32,
    renderer: R,
}

impl<'skip, R: Renderer> Horizontal<R> {
    pub fn new(renderer: R) -> Self {
        Self { y_anchor: 0.0, x_offset: 0.0, dim: ().into(), gap: 0.0, renderer }
    }
    pub fn add<W: Widget<'skip, R>, F: FnMut(W) -> WO, WO: Widget<'skip, R>>(mut self, mut f: F) -> Self {
        self.x_offset += self.gap;
        let mut w = f(W::new((self.x_offset, self.y_anchor), self.renderer));
        let size = w.size();
        self.x_offset += size.x;
        self.renderer = w.renderer();
        self
    }
    pub fn gap(mut self, gap: f32) -> Self {
        Self { y_anchor: self.y_anchor, x_offset: self.x_offset,dim: self.dim, gap, renderer: self.renderer }
    }

    pub fn padding<V: Into<Vec2<f32>>>(mut self, v: V) -> Self {
        let pad = v.into();
        self.y_anchor += pad.y;
        self.x_offset += pad.x;
        self
    }

    pub fn proc<PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>, P: Proc<'skip, Self, Out, R, Arg>, Out: Widget<'skip, R>, Arg>(self, proc: PA) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }


    pub fn iter<Iter: Iterator, F: FnMut(W, Iter::Item) -> W, W: Widget<'skip, R>>(mut self, mut items: Iter, mut f: F) -> Self {
        let mut w;
        for item in items.by_ref() {
            self.x_offset += self.gap;
            w = f(W::new((self.x_offset, self.y_anchor), self.renderer), item);
            let size = w.size();
            self.x_offset += size.x;
            self.renderer = w.renderer();
        }
        self
    }
}

impl<'skip,R: Renderer> Vertical<R> {  
    pub fn new(renderer: R) -> Self {
        Self { x_anchor: 0.0, y_offset: 0.0, dim: ().into(), gap: 0.0, renderer }
    }

    pub fn add<W: Widget<'skip, R>, F: FnMut(W) -> WO, WO: Widget<'skip, R>>(mut self, mut f: F) -> Self {
        self.y_offset += self.gap;
        let mut w = f(W::new((self.x_anchor, self.y_offset), self.renderer));
        let size = w.size();
        self.y_offset += size.y;
        self.renderer = w.renderer();
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        Self { x_anchor: self.x_anchor, y_offset: self.y_offset,dim: self.dim, gap, renderer: self.renderer }
    }

    pub fn padding<V: Into<Vec2<f32>>>(mut self, v: V) -> Self {
        let pad = v.into();
        self.x_anchor += pad.x;
        self.y_offset += pad.y;
        self
    }


    pub fn proc<PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>, P: Proc<'skip, Self, Out, R, Arg>, Out: Widget<'skip, R>, Arg>(self, proc: PA) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    pub fn iter<Iter: Iterator, F: FnMut(W, Iter::Item) -> W, W: Widget<'skip, R>>(mut self, mut items: Iter, mut f: F) -> Self {
        let mut w;
        for item in items.by_ref() {
            self.y_offset += self.gap;
            w = f(W::new((self.x_anchor, self.y_offset), self.renderer), item);
            let size = w.size();
            self.y_offset += size.y;
            self.renderer = w.renderer();
        }
        self
    }
}

impl<'skip, R: Renderer> Image<R> {
    pub fn image_id(mut self, img: ImageId) -> Self {
        self.widget.image_id = img;
        self
    }

    pub fn dim<V: Into<Vec2<f32>>>(mut self,dim: V) -> Self {
        self.widget.dim = dim.into();
        self
    }

    pub fn pos<V: Into<Vec2<f32>>>(mut self,pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }

    pub fn to_text<T: Into<TextW<'skip>>>(self, text: T) -> Text<'skip, R> {
        Text { 
            widget: text.into(), 
            renderer: self.renderer }
    }

    pub fn to_div<V: Into<DivW>>(self, dim: V) -> Div<R> {
        Div {
            widget: dim.into(), 
            renderer: self.renderer }
    }

    pub fn tint<C: Into<Color>>(mut self, color: C) -> Self {
        self.widget.tint = color.into();
        self
    }

    pub fn proc<PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>, P: Proc<'skip, Self, Out, R, Arg>, Out: Widget<'skip, R>, Arg>(self, proc: PA) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    pub fn horizontal<F: FnMut(Horizontal<R>) -> W, W:Widget<'skip,R>>(mut self, mut f: F) -> Self {
        let w = f(Horizontal { y_anchor: self.widget.pos.y, x_offset: self.widget.pos.x,dim: (&self.widget.dim).into(), gap: 0.0, renderer: self.renderer });
        self.renderer = w.renderer();
        self
    }

    pub fn vertical<F: FnMut(Vertical<R>) -> W, W:Widget<'skip,R>>(mut self, mut f: F) -> Self {
        let w = f(Vertical { x_anchor: self.widget.pos.x, y_offset: self.widget.pos.y, dim: (&self.widget.dim).into(), gap: 0.0, renderer: self.renderer });
        self.renderer = w.renderer();
        self
    }

}

impl<'skip,R: Renderer> Div<R> {
    pub fn pos<V: Into<Vec2<f32>>>(mut self,pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }
    
    pub fn dim<V: Into<Vec2<f32>>>(mut self,dim: V) -> Self {
        self.widget.dim = dim.into();
        self
    }

    pub fn to_text<T: Into<TextW<'skip>>>(self, text: T) -> Text<'skip, R> {
        Text { 
            widget: text.into(), 
            renderer: self.renderer }
    }
    
    pub fn to_img<I: Into<ImageW>>(self, img: I) -> Image<R> {
        Image { widget: img.into(), renderer: self.renderer }
    }

    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.widget.color = color.into();
        self
    }

    pub fn render(mut self) -> Self {
        self.renderer.render_div(&self.widget);
        self
    }

    pub fn on<F: FnMut(&mut DivW, &On)>(mut self, f: F) -> Self {
        self.renderer.on_div(&mut self.widget, f);
        self
    }

    pub fn key<F: FnMut(&mut DivW,  &Key)>(mut self, f: F) -> Self {
        self.renderer.key_div(&mut self.widget, f);
        self 
    }

    pub fn proc<PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>, P: Proc<'skip, Self, Out, R, Arg>, Out: Widget<'skip, R>, Arg>(self, proc: PA) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

    pub fn horizontal<F: FnMut(Horizontal<R>) -> W, W:Widget<'skip,R>>(mut self, mut f: F) -> Self {
        let w = f(Horizontal { y_anchor: self.widget.pos.y, x_offset: self.widget.pos.x,dim: (&self.widget.dim).into(), gap: 0.0, renderer: self.renderer });
        self.renderer = w.renderer();
        self
    }

    pub fn vertical<F: FnMut(Vertical<R>) -> W, W:Widget<'skip,R>>(mut self, mut f: F) -> Self {
        let w = f(Vertical { x_anchor: self.widget.pos.x, y_offset: self.widget.pos.y, dim: (&self.widget.dim).into(), gap: 0.0, renderer: self.renderer });
        self.renderer = w.renderer();
        self
    }

    pub fn clip<W: Widget<'skip, R>, F: FnMut(W) -> WO, WO: Widget<'skip,R>>(mut self, mut f: F) -> Self {
        self.renderer.start_clip(&self.widget);
        let w = f(W::new(&self.widget.pos, self.renderer)); 
        self.renderer = w.renderer();
        self.renderer.end_clip();    
        self
    }
}

impl<'skip, R: Renderer> Text<'skip, R> { 

    pub fn pos<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }

    pub fn to_div<V: Into<DivW>>(self, dim: V) -> Div<R> {
        Div {
            widget: dim.into(), 
            renderer: self.renderer }
    }

    pub fn to_img<I: Into<ImageW>>(self, img: I) -> Image<R> {
        Image { widget: img.into(), renderer: self.renderer }
    }


    pub fn text(mut self, text: &'skip str) -> Self {
        self.widget.text = text;
        self
    }

    pub fn font_id(mut self, font_id: usize) -> Self {
        self.widget.font_id = font_id;
        self
    }
    
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.widget.color = color.into();
        self
    }
    
    pub fn render(mut self) -> Self {
        self.renderer.render_text(&self.widget);
        self
    }

    pub fn proc<PA: Into<ProcArg<'skip, P, Self, Out, R, Arg>>, P: Proc<'skip, Self, Out, R, Arg>, Out: Widget<'skip, R>, Arg>(self, proc: PA) -> Out {
        let mut pa = proc.into();
        pa.proc.consume(self, pa.arg)
    }

}


pub trait Renderer {
    fn render_text<'skip>(&mut self, text: &TextW<'skip>);
    fn render_div(&mut self, div: &DivW);
    fn render_img(&mut self, img: &ImageW);
    fn on_img<F: FnMut(&mut ImageW, &On)>(&mut self,img: &mut ImageW, f: F);
    //fn on_text<'skip, F: FnMut(&mut TextW<'skip>, &On)>(&mut self,text: &mut TextW<'skip>, f: F); 
    fn on_div<F: FnMut(&mut DivW, &On)>(&mut self,div: &mut DivW, f: F);
    fn key_div<F: FnMut(&mut DivW, &Key)>(&mut self,div: &mut DivW, f: F);
    fn key_img<F: FnMut(&mut ImageW, &Key)>(&mut self,img: &mut ImageW, f: F); 
    //fn key_text<'skip, F: FnMut(&mut TextW<'skip>, &Key)>(&mut self,text: &mut TextW<'skip>, f: F);
    fn text_size<'skip>(&mut self, text: &TextW<'skip>) -> Vec2<f32>;
    fn start_clip(&mut self, dim: &DivW);
    fn end_clip(&mut self);
}

pub trait Proc<
    'skip, 
    In: Widget<'skip, R>, 
    Out: Widget<'skip, R>, 
    R: Renderer,
    Arg,
    > {
    fn consume(&mut self, widget: In, argv: Arg) -> Out;
}

pub(crate) struct ProcArg<'skip, P: Proc<'skip,In, Out, R, Arg>, In:Widget<'skip, R>, Out: Widget<'skip, R>, R:Renderer, Arg> {
    proc: P,
    arg: Arg,
    ph: PhantomData<(&'skip (), In, Out, Arg, R)>,
}

pub(crate) trait Widget<'skip, R: Renderer> {
    fn new<P: Into<Vec2<f32>>>(pos: P, renderer: R) -> Self;
    fn renderer(self) -> R;
    fn size(&mut self) -> Vec2<f32>; 
}

impl<'skip, R: Renderer> Widget<'skip, R> for Image<R> {
    fn new<P: Into<Vec2<f32>>>(pos: P, renderer: R) -> Self {
        Self { widget: ImageW { image_id: 0, pos: pos.into(), dim: ().into(), tint: ().into() }, renderer }
    }

    fn renderer(self) -> R {
        self.renderer
    }

    fn size(&mut self) -> Vec2<f32> {
        (&self.widget.dim).into()
    }
}

impl<'skip,R: Renderer> Widget<'skip, R> for Horizontal<R> {

   fn renderer(self) -> R {
       self.renderer
   }
   fn new<P: Into<Vec2<f32>>>(pos: P, renderer: R) -> Self {
       let p = pos.into();
       Self { y_anchor: p.y, x_offset: p.x, dim: ().into(), gap: 0.0, renderer }
   }
   fn size(&mut self) -> Vec2<f32> {
       (&self.dim).into()
   }
}


impl<'skip,R: Renderer> Widget<'skip, R> for Vertical<R> {
   fn renderer(self) -> R {
       self.renderer
   }
   fn new<P: Into<Vec2<f32>>>(pos: P, renderer: R) -> Self {
       let p = pos.into();
       Self { x_anchor: p.x, y_offset: p.y, dim: ().into(), gap: 0.0, renderer }
   }
   fn size(&mut self) -> Vec2<f32> {
       (&self.dim).into()
   }
}

impl<'skip, R: Renderer> Widget<'skip, R>  for Div<R> {
    fn renderer(self) -> R {
        self.renderer
    }
    fn new<P: Into<Vec2<f32>>>(pos: P, renderer: R) -> Self {
        let mut widget: DivW = ().into();
        widget.pos = pos.into();
        Self { widget, renderer }
    }

    fn size(&mut self) -> Vec2<f32> {
        (self.widget.dim.x, self.widget.dim.y).into()
    }

}

impl<'skip, R: Renderer> Widget<'skip, R>  for Text<'skip,R> {
    fn renderer(self) -> R {
        self.renderer
    }

    fn new<P: Into<Vec2<f32>>>(pos: P, renderer: R) -> Self {
        let mut widget: TextW<'_> = ().into();
        widget.pos = pos.into();
        Self { widget, renderer }
    }
    fn size(&mut self) -> Vec2<f32> {
        self.renderer.text_size(&self.widget)
    }
}

pub enum On {
    Press(Mouse),
    Release(Mouse),
    Hover(Vec2<f32>),
}

pub enum Mouse {
    Left,
    Right,
    Middle,
    Unknown,
}

pub enum Key {
    Press(&'static str),
    Release(&'static str)
}

pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<'skip, P: Proc<'skip,In, Out, R, Arg>, In:Widget<'skip, R>, Out: Widget<'skip, R>, R:Renderer, Arg> From<(P, Arg)> for ProcArg<'skip,P, In, Out,R, Arg> {
    fn from(value: (P, Arg)) -> Self {
        Self { proc: value.0, arg: value.1, ph: PhantomData::default() }
    }
}

impl<'skip, P: Proc<'skip,In, Out, R, ()>, In:Widget<'skip, R>, Out: Widget<'skip, R>, R:Renderer> From<(P)> for ProcArg<'skip,P, In, Out,R, ()> {
    fn from(value: (P)) -> Self {
        Self { proc: value, arg: (), ph: PhantomData::default() }
    }
}

impl From<(ImageId)> for ImageW {
    fn from(value: (ImageId)) -> Self {
       Self { image_id: value, pos: ().into(), dim: ().into(), tint: ().into() } 
    }
}

impl<C: Into<Color>> From<(ImageId, C)> for ImageW {
    fn from(value: (ImageId, C)) -> Self {
        Self { image_id: value.0, pos: ().into(), dim: ().into(), tint: value.1.into() }
    }
}

impl<'skip> From<()> for TextW<'skip> {
   fn from(_value: ()) -> Self {
       Self { text: "", font_id: 0, color: ().into(), pos: ().into() }
   } 
}


impl<'skip> From<(&'skip str)> for TextW<'skip> {
   fn from(value: (&'skip str)) -> Self {
       Self { text: value.into(), font_id: 0, color: ().into(), pos: ().into() }
   } 
}

impl<'skip, Pos: Into<Vec2<f32>>> From<(&'skip str, Pos)> for TextW<'skip> {
   fn from(value: (&'skip str, Pos)) -> Self {
        Self { text: value.0.into(), font_id: 0, color: ().into(), pos: value.1.into() }
   
    } 
}

impl<'skip> From<(&'skip str, Font)> for TextW<'skip> {
    fn from(value: (&'skip str, Font)) -> Self {
        Self { text: value.0, font_id: value.1, color: ().into(), pos: ().into() }
    }
}


impl<'skip, Col: Into<Color>, Pos: Into<Vec2<f32>>> From<(&'skip str, Font, Col, Pos)> for TextW<'skip> {
   fn from(value: (&'skip str, Font, Col, Pos)) -> Self {
        Self { text: value.0, font_id:value.1,color: value.2.into(), pos: value.3.into() }
   } 
}

impl From<()> for DivW  {
    fn from(_value: ()) -> Self {
        Self { 
            dim: ().into(), 
            rad: 0.0, 
            color: ().into(), 
            pos: ().into() }        
    }
}


impl<Dim: Into<Vec2<f32>>, Pos: Into<Vec2<f32>>> From<(Dim, Pos)> for DivW  {
    fn from(value: (Dim, Pos)) -> Self {
        Self { dim: value.0.into(), rad: 0.0, color: ().into(), pos: value.1.into() }
    }
}

impl<Dim: Into<Vec2<f32>>, Pos: Into<Vec2<f32>>, Col: Into<Color>> From<(Dim, Pos, f32, Col)> for DivW {
    fn from(value: (Dim, Pos, f32, Col)) -> Self {
        Self { dim: value.0.into(), rad: value.2, color: value.3.into(), pos: value.1.into() }
    }
}

impl<T> From<(T,T)> for Vec2<T> {
   fn from(value: (T,T)) -> Self {
        Self { x: value.0, y: value.1 }
    } 
}

impl<T: Copy> From<(&Vec2<T>)> for Vec2<T> {
    fn from(value: (&Vec2<T>)) -> Self {
        Self { x: value.x, y: value.y }
    }
}

impl<T: Default> From<()> for Vec2<T> {
   fn from(_value: ()) -> Self {
        Self { x: T::default(), y: T::default() }
    } 
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self { r: value.0, g: value.1, b: value.2, a: value.3 }
    }
}

impl From<()> for Color {
    fn from(_value: ()) -> Self {
        Self { r: 0, g: 0, b: 0, a: 0 }
    }
}


