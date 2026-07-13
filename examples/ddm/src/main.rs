use std::time::Duration;

use skip::{Div, Mouse, On, Text};
use skip_skia::{AppController, DrawFn, UserEvent};

enum Message {}

impl UserEvent for Message {}

struct App {
    main_window: winit::window::WindowId,
    shared: SharedRes,
}
enum Color {
    Background,
    UiBg
}

impl From<Color> for skip::Color {
    #[inline]
    fn from(value: Color) -> Self {
        match value {
            Color::Background => (16, 20, 28, 1).into(),
            Color::UiBg => (20, 24, 33, 1).into()
        }
    }
}
impl App {
    fn draw(
        shared: &mut SharedRes,
        mut ui: skip::Horizontal<skip_skia::Canvas>,
        proxy: &winit::event_loop::EventLoopProxy<Message>,
    ) -> Option<Duration> {
        let win = ui.canvas_size();
        ui.add(|background: Div<_>| {
            background
            .size((&win))
            .color(Color::Background)
            .render()
            .horizontal(|layout| { 
                layout
                .gap(5.0)
                .iter((0..10, 3), |button: Div<_>, _| {
                    button
                    .color(Color::UiBg)
                    .size((100.0, 50.0))
                    .render()
                })
            })
        });
        Some(Duration::from_millis(16))
    }
}

struct SharedRes;

impl AppController<Message, SharedRes> for App {
    fn bootstrap<'skip>(&mut self, mut context: skip_skia::Context<'skip, SharedRes, Message>) {
        self.main_window = context.new_window(
            winit::window::WindowAttributes::default(),
            DrawFn(App::draw),
            None,
            );
        context.set_visible(&self.main_window, true);
    }
    fn user_event<'skip>(
        &mut self,
        user_event: Message,
        context: skip_skia::Context<'skip, SharedRes, Message>,
    ) {
    }
    fn share_resource(&mut self) -> &mut SharedRes {
        &mut self.shared
    }
}

fn main() {
    skip_skia::run_app(App {
        main_window: winit::window::WindowId::dummy(),
        shared: SharedRes,
    });
    println!("hello world!")
}
