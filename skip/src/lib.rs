pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct Text<'skip, R: Renderer> {
    widget: TextW<'skip>,
    renderer: R,
}

pub struct TextW<'skip> {
    pub text: &'skip str,
    pub font_id: Font,
    pub size: usize,
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

pub struct Child<R> {
    pos: Vec2<f32>,
    renderer: R,
}

impl<'skip> From<()> for TextW<'skip> {
   fn from(_value: ()) -> Self {
       Self { text: "", font_id: 0, size: 0, color: ().into(), pos: ().into() }
   } 
}


impl<'skip> From<(&'skip str)> for TextW<'skip> {
   fn from(value: (&'skip str)) -> Self {
       Self { text: value.into(), font_id: 0, size: 0, color: ().into(), pos: ().into() }
   } 
}

impl<'skip, Pos: Into<Vec2<f32>>> From<(&'skip str, Pos)> for TextW<'skip> {
   fn from(value: (&'skip str, Pos)) -> Self {
        Self { text: value.0.into(), font_id: 0, size: 0, color: ().into(), pos: value.1.into() }
   
    } 
}

impl<'skip> From<(&'skip str, Font, usize)> for TextW<'skip> {
    fn from(value: (&'skip str, Font, usize)) -> Self {
        Self { text: value.0, font_id: value.1, size: value.2, color: ().into(), pos: ().into() }
    }
}


impl<'skip, Col: Into<Color>, Pos: Into<Vec2<f32>>> From<(&'skip str, Font, usize, Col, Pos)> for TextW<'skip> {
   fn from(value: (&'skip str, Font, usize, Col, Pos)) -> Self {
        Self { text: value.0, font_id:value.1, size: value.2, color: value.3.into(), pos: value.4.into() }
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

impl<'skip,R: Renderer> Div<R> {
    pub fn new<Div: Into<DivW>>(widget: Div, renderer: R) -> Self {
        Self { 
            widget: widget.into(),
            renderer 
        }
    }

    pub fn turn<Div: Into<DivW>>(self, widget: Div) -> Self {
        Self { widget: widget.into(), renderer: self.renderer }
    }

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

    pub fn proc<P: Proc<'skip, Self, Out, R>, Out: Widget<'skip, R>>(self, mut p: P) -> Out {
        p.consume(self)
    }

    pub fn children<F: FnMut(Child<R>) -> O, O: Widget<'skip, R>>(mut self, mut f: F) -> Self {
        let w = f(Child { pos: (self.widget.pos.x, self.widget.pos.y).into(), renderer: self.renderer });
        self.renderer = w.renderer();
        self
    }
}

impl<'skip, R: Renderer> Text<'skip, R> {

    pub fn new<T: Into<TextW<'skip>>>(text: T,renderer: R) -> Self {
       Self {
           widget: text.into(),
           renderer
       } 
    }

    pub fn turn<T: Into<TextW<'skip>>>(self, widget: T) -> Self {
        Self { widget: widget.into(), renderer: self.renderer }
    }


    pub fn pos<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.widget.size = size;
        self
    }

    pub fn to_div<V: Into<DivW>>(self, dim: V) -> Div<R> {
        Div {
            widget: dim.into(), 
            renderer: self.renderer }
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
    
    pub fn on<F: FnMut(&mut TextW<'skip>, &On)>(mut self, f: F) -> Self {
        self.renderer.on_text(&mut self.widget,f);
        self
    }
    
    pub fn key<F: FnMut(&mut TextW,  &Key)>(mut self, f: F) -> Self {
        self.renderer.key_text(&mut self.widget, f);
        self
 
    }
    pub fn render(mut self) -> Self {
        self.renderer.render_text(&self.widget);
        self
    }

    pub fn proc<P: Proc<'skip, Self, Out, R>, Out: Widget<'skip, R>>(self, mut p: P) -> Out {
        p.consume(self)
    }

}

impl<R: Renderer> Child<R> {
    pub fn text<'skip,F: FnMut(Text<'skip, R>) -> O, O: Widget<'skip, R>>(mut self, mut f: F) -> Self {
        let o = f(Text { widget: ("", (self.pos.x, self.pos.y)).into(), renderer: self.renderer });
        self.renderer = o.renderer();
        self
    }

    pub fn div<'skip,F: FnMut(Div<R>) -> O, O: Widget<'skip, R>>(mut self, mut f: F) -> Self {
        let o = f(Div { widget: ((), (self.pos.x, self.pos.y)).into(),renderer: self.renderer});
        self.renderer = o.renderer();
        self
    }
}

pub trait Renderer {
    fn render_text<'skip>(&mut self, text: &TextW<'skip>);
    fn render_div(&mut self, div: &DivW);
    fn on_text<'skip, F: FnMut(&mut TextW<'skip>, &On)>(&mut self,text: &mut TextW<'skip>, f: F); 
    fn on_div<F: FnMut(&mut DivW, &On)>(&mut self,div: &mut DivW, f: F);
    fn key_div<F: FnMut(&mut DivW, &Key)>(&mut self,div: &mut DivW, f: F);
    fn key_text<'skip, F: FnMut(&mut TextW<'skip>, &Key)>(&mut self,text: &mut TextW<'skip>, f: F);
}

pub(crate) trait Widget<'skip, R: Renderer> {
    fn renderer(self) -> R;
}

impl<'skip, R: Renderer> Widget<'skip, R>  for Div<R> {
    fn renderer(self) -> R {
        self.renderer
    }
}

impl<'skip, R: Renderer> Widget<'skip, R>  for Text<'skip,R> {
    fn renderer(self) -> R {
        self.renderer
    }
}

impl<'skip, R: Renderer> Widget<'skip, R> for Child<R> {
    fn renderer(self) -> R {
        self.renderer
    }
}

pub trait Proc<'skip, In: Widget<'skip, R>, Out: Widget<'skip, R>, R: Renderer> {
    fn consume(&mut self, widget: In) -> Out;
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
