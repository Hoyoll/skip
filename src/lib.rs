pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}


pub struct Text<'skip, R: Renderer<'skip>> {
    pub text: &'skip str,
    pub font_id: usize,
    pub size: usize,
    pub color: [usize;4], 
    renderer: R,
}


pub struct Div<R> {
    pub dim: Vec2<f32>,
    pub rad: f32,
    pub color: [usize;4],
    renderer: R,
}

impl<'skip,R: Renderer<'skip>> Div<R> {
    pub fn to_text(self, text: &'skip str) -> Text<'skip, R> {
        Text { text, font_id: 0, size: 0, color: self.color, renderer: self.renderer }
    }
    pub fn color(&mut self, color: [usize;4]) -> &mut Self {
        self.color = color;
        self
    }

    pub fn render(&mut self) {
        self.renderer.render_div();
    }

    pub fn on<F: FnMut(&mut Self, On)>(&mut self, f: F) {
        self.renderer.on_div(f);
    }
}

impl<'skip, R: Renderer<'skip>> Text<'skip, R> {
    pub fn to_div(self, dim: Vec2<f32>) -> Div<R> {
        Div {dim, rad: 0.0, color: [0,0,0,0], renderer: self.renderer}
    }

    pub fn text(&mut self, text: &'skip str) -> &mut Self {
        self.text = text;
        self
    }

    pub fn font_id(&mut self, font_id: usize) -> &mut Self {
        self.font_id = font_id;
        self
    }
    pub fn color(&mut self, color: [usize;4]) -> &mut Self {
        self.color = color;
        self
    }
    pub fn on<F: FnMut(&mut Self, On)>(&mut self, f: F) {
        self.renderer.on_text(f);
    }
}

pub trait Renderer<'skip> {
    fn render_text(&self);
    fn render_div(&self);
    fn on_text<F: FnMut(&mut Text<'skip, Self>, On)>(&self, f: F);
    fn on_div<F: FnMut(&mut Div<Self>, On)>(&self, f: F);
}

pub enum On {
    Press,
    Release,
    Hover,
    Key(&'static str),
}

pub fn lib() {
    println!("Hello, world!");
}
