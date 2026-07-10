use std::time::Duration;

use skip::{Div, Mouse, On, Text};
use skip_skia::{AppController, DrawFn, UserEvent};

enum Message {}

impl UserEvent for Message {}

struct App {
    main_window: winit::window::WindowId,
    shared: SharedRes,
}

impl App {
    fn draw(
        shared: &mut SharedRes,
        ui: skip::Horizontal<skip_skia::Canvas>,
        proxy: &winit::event_loop::EventLoopProxy<Message>,
    ) -> Option<Duration> {
        ui.gap(50.0).iter(0..10_000, |div: Div<_>, i| {
            div.color((255, 255, 255, 255))
                .size((100.0, 50.0))
                .hover(|coords, w| {
                    w.color((255, 0, 0, 255)).rad(50.0).child(|div: Div<_>| {
                        div.padding((0.0, 100.0)).color((0, 255, 0, 255)).render()
                    })
                })
                .hover(|coords, div| div)
                .render()
                .on(|on| match on {
                    On::Press(Mouse::Left) => println!("pressed"),
                    On::Release(Mouse::Left) => println!("released"),
                    _ => (),
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
