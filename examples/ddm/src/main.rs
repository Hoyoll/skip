use skip_skia::AppController;
use winit::window::{WindowAttributes, WindowId};

struct App {
    id: WindowId,
    player: Player,
}

enum Music {

}

struct Player {}

impl AppController<Music> for App {
    fn on_draw(
        &mut self,
        on_window: winit::window::WindowId,
        layout: skip::Horizontal<skip_skia::Canvas>,
    ) -> Option<std::time::Duration>
    {
       None 
    }

    fn bootstrap<'skip>(&mut self, mut context: skip_skia::Context<'skip>) {
        let attr = WindowAttributes::default()
            .with_resizable(false)
            .with_title("DDM!");
        self.id = context.new_window(attr);
    }
}

fn main() {
}
