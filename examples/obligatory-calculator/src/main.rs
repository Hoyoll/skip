use std::time::Duration;

use skip::{Circle, Cursor, Div, Font, Horizontal, Mouse, On, Proc, State, Text};
use skip_skia::{AppController, Canvas, DrawFn, Event, UserEvent};

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

impl UserEvent for Message {}

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
    Button::Action("_"), Button::Number("0"),Button::Action("="), Button::Operand("/")
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
        .render()
        .child(|text: Text<_>| {
            text
            .font_id(argv)

            .size(80.0f32)
            .color(Color::Light)
            .text(&self.text)
            .render()
        })
   } 
}

impl Calc {
    fn main_window(context: &mut Context, mut ui: Horizontal<Canvas>, proxy: &Event<Message>) -> Option<Duration> {
        let win = ui.canvas_size();
        let gap = 5.0;
        let height = win.y / 5.0 - gap;
        let width = win.x / 4.0 - gap;
        ui
        //.cursor(Cursor::Default)
        .add(|background: Div<_>| {
            background
            .size(&win)
            .color(Color::Background)
            .render()
            .hover(|pos, background| {
                background
                .child(|circle: Circle<_>| {
                    circle
                    .pos(&pos)
                    .radius(height / 2.0)
                    .color(Color::Light)
                    .render()
                })
            })
            .vertical(|layout| {
                layout
                .padding((gap / 2.0, 0.0))
                .gap(gap)
                .add(|text_box: Div<_>| {
                    text_box
                    .size((win.x - gap, height))
                    .color(Color::UiBg)
                    .proc((&mut context.text_input, context.fira_code))
                })
                .add(|layout: skip::Horizontal<_>| {
                    layout
                    .gap(gap)
                    .iter((BUTTONS.iter(), 4), |button: Div<_>, btn| {
                        let mut text_color = Color::Fg;
                        button
                        .color(Color::UiBg)
                        .size((width, height))
                        //.proc((skip_cn::Border(Color::UiBg, (10.0, 10.0), (0.0, 0.0))))
                        .render()
                        .on(|on| {
                           match on {
                               (Mouse::Left, State::Pressed) => context.text_input.accept(btn),
                                _ => ()
                           } 
                        })
                        //.cursor(Cursor::Default)
                        .hover(|_,div| {
                            text_color = Color::Light;
                            //div.cursor(Cursor::Pointer)
                            div
                        })  
                        .child(|label: Text<_>| {
                            let pad_x = (width / 2.0) - (width / 15.0);
                            let pad_y = height / 2.0 - height / 4.0; 
                            //let pad = (pad_x,pad_y); 
                            label
                            .padding((pad_x, pad_y))
                            .color(&text_color)
                            .size(40.0f32)
                            .font_id(context.fira_code)
                            .text(btn.get_text())
                            .render()
                        })
                    })
                })
            })
        });
        Some(Duration::from_millis(16))
    }
}

struct Context {
    title: String,
    fira_code: Font,
    text_input: TextInput,
}

impl AppController<Message, Context> for Calc {
   fn bootstrap(&mut self, mut context: skip_skia::Context<Context, Message>) {
       let id = context.new_window(winit::window::WindowAttributes::default(), DrawFn(Calc::main_window), None);
       context.request_redraw(&id);
       context.set_visible(&id, true);

       self.context.fira_code = context.new_font(FIRA_CODE, None).unwrap();
   }

   fn user_event(&mut self, user_event: Message, context: skip_skia::Context<Context, Message>) {

   }
   fn share_resource(&mut self) -> &mut Context {
       &mut self.context
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
