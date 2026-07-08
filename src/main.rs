use std::time::Duration;

use skip::{Div, Text};
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
        ui.on(|l,on| {
            Some(|l| {
                l
            })      
        });
        ui.gap(50.0).iter(0..100_000_000, |div: Div<_>, i| {
            div.color((255, 255, 255, 255))
                .dim((100.0, 50.0))
                .on(|d, on| {
                    match on {
                        skip::On::Hover(_) => {
                            //           println!("hover!");
                            d.color = (255, 0, 0, 255).into()
                        }
                        skip::On::Press(skip::Mouse::Left) => d.color = (0, 255, 0, 255).into(),
                        _ => (),
                    }
                })
                .render()
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
