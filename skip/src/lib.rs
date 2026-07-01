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
    pub font_id: usize,
    pub size: usize,
    pub color: Color,
    pub pos: Vec2<f32>,
}

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

impl<T> From<(T,T)> for Vec2<T> {
   fn from(value: (T,T)) -> Self {
        Self { x: value.0, y: value.1 }
    } 
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self { r: value.0, g: value.1, b: value.2, a: value.3 }
    }
}

impl<'skip,R: Renderer> Div<R> {
    pub fn new(renderer: R) -> Self {
        Self { 
            widget: DivW {
                dim: (0.0, 0.0).into(), 
                rad: 0.0, 
                color: (0,0,0,0).into(),
                pos: (0.0, 0.0).into()
        }, renderer }
    }

    pub fn pos<V: Into<Vec2<f32>>>(mut self,pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }

    
    pub fn dim<V: Into<Vec2<f32>>>(mut self,dim: V) -> Self {
        self.widget.dim = dim.into();
        self
    }
    pub fn to_text(self, text: &'skip str) -> Text<'skip, R> {
        Text { 
            widget: TextW { 
                text, 
                font_id: 0, 
                size: 0, 
                color: self.widget.color,
                pos: Vec2 { x: 0.0, y: 0.0 }
            }, 
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


}

impl<'skip, R: Renderer> Text<'skip, R> {

    pub fn new(renderer: R, text: &'skip str) -> Self {
       Self {
           widget: TextW {
                text,
                font_id: 0,
                color: (0,0,0,0).into(),
                size: 0,
                pos: Vec2 { x: 0.0, y: 0.0 }
           },
           renderer
       } 
    }

    pub fn pos<V: Into<Vec2<f32>>>(mut self, pos: V) -> Self {
        self.widget.pos = pos.into();
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.widget.size = size;
        self
    }

    pub fn to_div<V: Into<Vec2<f32>>>(self, dim: V) -> Div<R> {
        Div {
            widget: DivW {
                dim: dim.into(),
                pos: (0.0, 0.0).into(),
                rad: 0.0, 
                color: (0,0,0,0).into()
            }, 
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

pub trait Renderer {
    fn render_text<'skip>(&mut self, text: &TextW<'skip>);
    fn render_div(&mut self, div: &DivW);
    fn on_text<'skip, F: FnMut(&mut TextW<'skip>, &On)>(&mut self,text: &mut TextW<'skip>, f: F); 
    fn on_div<F: FnMut(&mut DivW, &On)>(&mut self,div: &mut DivW, f: F);
    fn key_div<F: FnMut(&mut DivW, &Key)>(&mut self,div: &mut DivW, f: F);
    fn key_text<'skip, F: FnMut(&mut TextW<'skip>, &Key)>(&mut self,text: &mut TextW<'skip>, f: F);
}

pub trait Widget<'skip, R: Renderer> {}

impl<'skip, R: Renderer> Widget<'skip, R>  for Div<R> {}

impl<'skip, R: Renderer> Widget<'skip, R>  for Text<'skip,R> {}

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
