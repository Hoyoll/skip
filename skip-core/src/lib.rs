pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

pub struct Text<'skip, R: Renderer<'skip>> {
    widget: TextW<'skip>,
    renderer: R,
}

pub struct TextW<'skip> {
    pub text: &'skip str,
    pub font_id: usize,
    pub size: usize,
    pub color: [usize;4],
    pub pos: Vec2<f32>,
}

pub struct Div<R> {
    widget: DivW,
    renderer: R,
}

pub struct DivW {
    pub dim: Vec2<f32>,
    pub rad: f32,
    pub color: [usize;4],
    pub pos: Vec2<f32>,
 
}

impl<'skip,R: Renderer<'skip>> Div<R> {
    pub fn new(renderer: R) -> Self {
        Self { 
            widget: DivW {
                dim: Vec2 {x: 0.0, y: 0.0}, 
                rad: 0.0, 
                color: [0,0,0,0],
                pos: Vec2 { x: 0.0, y: 0.0 }
        }, renderer }
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
    pub fn color(mut self, color: [usize;4]) -> Self {
        self.widget.color = color;
        self
    }

    pub fn render(self) -> Self {
        self.renderer.render_div(&self.widget);
        self
    }

    pub fn on<F: FnMut(&mut DivW, On)>(mut self, f: F) -> Self {
        self.renderer.on_div(&mut self.widget, f);
        self
    }
}

impl<'skip, R: Renderer<'skip>> Text<'skip, R> {

    pub fn new(renderer: R, text: &'skip str) -> Self {
       Self {
           widget: TextW {
                text,
                font_id: 0,
                color: [0,0,0,0],
                size: 0,
                pos: Vec2 { x: 0.0, y: 0.0 }
           },
           renderer
       } 
    }
    pub fn to_div(self, dim: Vec2<f32>) -> Div<R> {
        Div {
            widget: DivW {
                dim,
                pos: Vec2 { x: 0.0, y: 0.0 },
                rad: 0.0, 
                color: [0,0,0,0]
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
    pub fn color(mut self, color: [usize;4]) -> Self {
        self.widget.color = color;
        self
    }
    pub fn on<F: FnMut(&mut TextW<'skip>, On)>(mut self, f: F) -> Self {
        self.renderer.on_text(&mut self.widget,f);
        self
    }

    pub fn render(self) -> Self {
        self.renderer.render_text(&self.widget);
        self
    }

}

pub trait Renderer<'skip> {
    fn render_text(&self, text: &TextW<'skip>);
    fn render_div(&self, div: &DivW);
    fn on_text<F: FnMut(&mut TextW<'skip>, On)>(&self,text: &mut TextW<'skip>, f: F); 
    fn on_div<F: FnMut(&mut DivW, On)>(&self,div: &mut DivW, f: F);
}

pub enum On {
    Press,
    Release,
    Hover,
    Key(&'static str),
}
