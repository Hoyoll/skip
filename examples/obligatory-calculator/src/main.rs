use std::time::Duration;

use skip::{Center, Circle, Div, End, Font, Horizontal, Hover, Keys, Mouse, Plain, Proc, State, Text, Vec2, Vertical, X, XY, Y};
use skip_skia::{AppController, Canvas, Event};

enum Color {
    Background,
    UiBg,
    Fg,
    Light,
}

impl From<Color> for skip::Color {
    #[inline]
    fn from(value: Color) -> Self {
        match value {
            Color::Background => (16, 20, 28, 255).into(),
            Color::UiBg => (20, 24, 33, 255).into(),
            Color::Fg => (90, 99, 120, 255).into(),
            Color::Light => (191, 189, 182, 255).into()
        }
    }
}

impl From<&Color> for skip::Color {
    #[inline]
    fn from(value: &Color) -> Self {
        match value {
            Color::Background => (16, 20, 28, 255).into(),
            Color::UiBg => (20, 24, 33, 255).into(),
            Color::Fg => (90, 99, 120, 255).into(),
            Color::Light => (191, 189, 182, 255).into()
        }
    }
}

enum Message {}
struct Calc {
    context: Context,
}

const FIRA_CODE: &[u8] = include_bytes!("../assets/fonts/fira_code.ttf");

enum Button {
    Number(&'static str),
    Operand(&'static str),
    Action(&'static str)
}

impl Button {
    pub fn get_text(&self) -> &'static str {
        match self {
           Button::Action(text) => text,
           Button::Number(num) => num,
           Button::Operand(op) => op
        }
    }
}

const BUTTONS: [Button; 16] = [
    Button::Number("1"), Button::Number("2"), Button::Number("3"), Button::Operand("+"),
    Button::Number("4"), Button::Number("5"), Button::Number("6"), Button::Operand("-"),
    Button::Number("7"), Button::Number("8"), Button::Number("9"), Button::Operand("*"),
    Button::Action("<="), Button::Number("0"),Button::Action("="), Button::Operand("/")
];

struct TextInput {
    pub text: String,
}

impl TextInput {
    pub fn accept(&mut self, button: &Button) {
        match button {
            Button::Number(num) | Button::Operand(num) => {
                self.text.push_str(num);
            },
            Button::Action("=") => {
                match meval::eval_str(&self.text) {
                    Ok(val) => {
                        self.text = val.to_string();
                    },
                    Err(_) => {}
                }
            },
            Button::Action("_") => {
                self.text.clear();
            },
            _ => ()
        }
    }
}

impl<'skip> Proc<'skip, Canvas<'skip>> for &mut TextInput {
   type Widget = Div<Canvas<'skip>>;
   type Arg = Font;

   fn consume(self, widget: Self::Widget, argv: Self::Arg) -> Self::Widget {
       widget
        .render::<Plain<_>>(Color::UiBg) 
        .child(|text: Text<_>| {
            text
            .size(80.0f32)
            .font_id(argv)
            .text(&self.text)
            //.child(|div: Div<_>| {
            //    div
            //    .proc(BORDER)
            //    .render::<Plain<_>>(Color::UiBg) 
            //})
            .align::<End, X>()
            .align::<Center, Y>()
            .render(Color::Light)

        })
   } 
}

const BORDER: skip::cn::Border<Color, (f32, f32), (f32, f32)> = skip::cn::Border(Color::Fg, (1.0, 1.0), (0.0, 0.0));
struct Context {
    title: String,
    fira_code: Font,
    text_input: TextInput,
}

impl AppController<Message> for Calc {
   fn bootstrap(&mut self, mut context: skip_skia::Context) {
       let attr = winit::window::WindowAttributes::default().with_title(self.context.title.clone());
       let id = context.new_window(attr);
       context.request_redraw(&id);
       context.set_visible(&id, true);

       self.context.fira_code = context.new_font(FIRA_CODE, None).unwrap();
   }

   fn on_user_event<'skip>(&mut self, user_event: Message, context: skip_skia::Context<'skip>) {
       
   }

   fn on_draw(
       &mut self,
       on_window: winit::window::WindowId,
       mut ui: skip::Horizontal<Canvas>,
   ) -> Option<Duration>
   {
        let win = ui.canvas_size();
        let gap = 5.0;
        let height = win.y / 5.0 - gap;
        let width = win.x / 4.0 - gap;
        ui
        .add(|background: Div<_>| {
            background
            .size(&win)
            .render::<Plain<_>>(Color::Background)
            .on::<Hover,_>(|(pos, background)| {
                background
                .child(|circle: Circle<_>| {
                    circle
                    .pos(&pos)
                    .radius(height / 2.0)
                    .render(Color::Light)
                })
            })
            .child(|layout: Vertical<_>| {
                layout
                .padding((gap / 2.0, 1.0))
                .gap(gap)
                .add(|text_box: Div<_>| {
                    text_box
                    .size((win.x - gap, height))
                    .proc(BORDER)
                    .proc((&mut self.context.text_input, self.context.fira_code))
                })
                .add(|layout: skip::Horizontal<_>| {
                    layout
                    .gap(gap)
                    .iter((BUTTONS.iter(), 4), |button: Div<_>, btn| {
                        let mut text_color = Color::Fg;
                        button
                        .size((width, height))
                        .proc(BORDER)
                        .render::<Plain<_>>(Color::UiBg)
                        .on::<Keys,_>(|on| {
                           match on {
                               (Mouse::Left, State::Pressed) => self.context.text_input.accept(btn),
                                _ => ()
                           } 
                        })
                        .on::<Hover,_>(|(_, div)| {
                            text_color = Color::Light;
                            div
                        })  
                        .child(|label: Text<_>| {
                            label
                            .size(50.0f32)
                            .font_id(self.context.fira_code)
                            .text(btn.get_text())
                            .align::<Center, XY>()
                            //.child(|div: Div<_>| {
                            //    div
                            //    .proc(BORDER)
                            //    .render::<Plain<_>>(Color::UiBg) 
                            //})
                            .render(&text_color)
                       })
                    })
                })
            })
        });
        Some(Duration::from_millis(16))
 
   }

   fn on_key(&mut self, _on_window: winit::window::WindowId, _key: (skip::Key, skip::State)) {
       
   }
}

fn main() {
    let calc = Calc { context: Context {
        title: String::from("Calculator!"),
        fira_code: Font::default(),
        text_input: TextInput { text: String::new() }
    } };
    skip_skia::run_app(calc);
    //println!("Hello, world!");
}
