use std::time::Duration;

use skip::{Div, Font, Horizontal, Text};
use skip_skia::{AppController, Canvas, DrawFn, UserEvent};
use winit::{event_loop::EventLoopProxy, window::WindowAttributes};

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

enum Message {
    
}

impl UserEvent for Message {}

struct Calc {
    context: Context,
}

const FIRA_CODE: &[u8] = include_bytes!("../assets/fonts/fira_code.ttf");

impl Calc {
    fn main_window(context: &mut Context, mut ui: Horizontal<Canvas>, proxy: &EventLoopProxy<Message>) -> Option<Duration> {
        let win = ui.canvas_size();
        let gap = 5.0;
        let height = win.y / 5.0 - gap;
        let width = win.x / 4.0 - gap;
        ui.add(|background: Div<_>| {
            background
            .size((&win))
            .color(Color::Background)
            .render()
            .vertical(|layout| {
                layout
                .padding((gap, 0.0))
                .gap(gap)
                .add(|text_box: Div<_>| {
                    text_box
                    .size((win.x - gap, height))
                    .color(Color::UiBg)
                    .render()
                })
                .add(|layout: Horizontal<_>| { 
                    layout
                    .gap(gap)
                    .iter((
                        [
                        "1", "2", "3","+", 
                        "4", "5", "6", "-", 
                        "7", "8", "9", ":",
                        "", "0", "=", "*"
                        ].iter(), 4), 
                        |button: Div<_>, number| {
                        button
                        .color(Color::UiBg)
                        .size((width, height))
                        .hover(|_, button| {
                            button.child(|background: Div<Canvas<'_>>| {
                                background
                                .padding((-gap, -gap))
                                .size((width + gap * 2.0, height + gap * 2.0))
                                .color(Color::Light)
                                .render()
                            })
                        })      
                        .render()
                        .child(|label: Text<_>| {
                            let pad_x = (width / 2.0) - (width / 15.0);
                            let pad_y = height / 2.0 + (height / 10.0);
                            label
                            .padding((pad_x, pad_y))
                            .color(Color::Fg)
                            .font_id(context.fira_code)
                            .text(*number)
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
}

impl AppController<Message, Context> for Calc {
   fn bootstrap(&mut self, mut context: skip_skia::Context<Context, Message>) {
       let id = context.new_window(WindowAttributes::default(), DrawFn(Calc::main_window), None);
       context.request_redraw(&id);
       context.set_visible(&id, true);

       self.context.fira_code = context.new_font(FIRA_CODE, 40.0, None).unwrap();
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
        fira_code: Font::default()
    } };
    skip_skia::run_app(calc);
    //println!("Hello, world!");
}
