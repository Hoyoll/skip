use std::{
    collections::HashMap,
    ffi::CString,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use glutin::{
    context::NotCurrentGlContext,
    display::{GetGlDisplay, GlDisplay},
    surface::GlSurface,
};
use raw_window_handle::HasWindowHandle;

pub fn run_app<Shared, App: AppController<Event, Shared>, Event: UserEvent + 'static>(app: App) {
    let event_loop: winit::event_loop::EventLoop<Event> =
        winit::event_loop::EventLoop::with_user_event()
            .build()
            .unwrap();
    let mut wn = WinitRenderer {
        on: Vec::new(),
//        key: Vec::new(),
        mouse_pos: (0.0, 0.0).into(),
        current_focus: winit::window::WindowId::dummy(),
        app,
        windows: HashMap::new(),
        proxy: event_loop.create_proxy(),
        paint: skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.0), None),
        fonts: Vec::new(),
        images: Vec::new(),
        font_mgr: skia_safe::FontMgr::new(),
    };

    event_loop.run_app(&mut wn);
}

pub trait UserEvent {}

pub struct DrawFn<Shared, T: UserEvent + 'static>(
    pub fn(
        &mut Shared,
        skip::Horizontal<Canvas>,
        &winit::event_loop::EventLoopProxy<T>,
    ) -> Option<Duration>,
);

pub struct KeyFn<Shared, T: UserEvent + 'static>(
    pub fn(&mut Shared, skip::Key, &winit::event_loop::EventLoopProxy<T>)
    );

pub trait AppController<T: UserEvent, Shared> {
    fn bootstrap<'skip>(&mut self, context: Context<'skip, Shared, T>);
    fn user_event<'skip>(&mut self, user_event: T, context: Context<'skip, Shared, T>);
    fn share_resource(&mut self) -> &mut Shared;
}

pub enum Redraw {
    FocusOnly,
    Always,
}

struct Window<Shared, T: UserEvent + 'static> {
    window: winit::window::Window,
    surface: skia_safe::Surface,
    dr_context: skia_safe::gpu::DirectContext,
    skia_context: glutin::context::PossiblyCurrentContext,
    fb_info: skia_safe::gpu::gl::FramebufferInfo,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    draw_fn: DrawFn<Shared, T>,
    key_fn: Option<KeyFn<Shared, T>>,
    next_redraw: Option<Instant>,
    redraw_policy: Redraw,
    //focused: bool,
}

pub struct Canvas<'skip> {
    on: &'skip Vec<skip::On>,
    mouse_pos: &'skip skip::Vec2<f32>,
    //key: &'skip Vec<skip::Key>,
    canvas: &'skip skia_safe::Canvas,
    paint: &'skip mut skia_safe::Paint,
    fonts: &'skip Vec<skia_safe::Font>,
    images: &'skip Vec<skia_safe::Image>,
    window_dim: skip::Vec2<f32>,
}

impl<'a> skip::Renderer for Canvas<'a> {
    fn canvas_size(&mut self) -> skip::Vec2<f32> {
        (&self.window_dim).into()
    }
    fn render_div(&mut self, div: &skip::DivW) {
        let right = div.pos.x + div.size.x;
        let bottom = div.pos.y + div.size.y;

        if right <= 0.0
            || bottom <= 0.0
            || div.pos.x >= self.window_dim.x
            || div.pos.y >= self.window_dim.y
        {
            //println!("skipped!");
            return;
        }
        //dbg!(div);
        //println!("draw!");
        self.paint
            .set_argb(div.color.a, div.color.r, div.color.g, div.color.b);
        self.canvas.draw_round_rect(
            skia_safe::Rect::from_xywh(div.pos.x, div.pos.y, div.size.x, div.size.y),
            div.rad,
            div.rad,
            self.paint,
        );
    }

    //    #[inline]
    fn text_size<'skip>(&mut self, text: &skip::TextW<'skip>) -> skip::Vec2<f32> {
        let fonts = &self.fonts[text.font_id];
        let (_, rect) = fonts.measure_str(text.text, None);
        (rect.width(), rect.height()).into()
    }

    fn render_text<'skip>(&mut self, text: &skip::TextW<'skip>) {
        if text.pos.x >= self.window_dim.x || text.pos.y >= self.window_dim.y {
            return;
        }
        self.paint
            .set_argb(text.color.a, text.color.r, text.color.g, text.color.b);
        self.canvas.draw_str(
            text.text,
            (text.pos.x, text.pos.y),
            &self.fonts[text.font_id],
            self.paint,
        );
    }

    fn render_img(&mut self, img: &skip::ImageW) {
        let right = img.pos.x + img.size.x;
        let bottom = img.pos.y + img.size.y;

        if right <= 0.0
            || bottom <= 0.0
            || img.pos.x >= self.window_dim.x
            || img.pos.y >= self.window_dim.y
        {
            return;
        }
        match self.images.get(img.image_id) {
            Some(image) => {
                self.paint
                    .set_argb(img.tint.a, img.tint.r, img.tint.g, img.tint.b);
                self.canvas.draw_image_rect(
                    image,
                    None,
                    skia_safe::Rect::from_xywh(img.pos.x, img.pos.y, img.size.x, img.size.y),
                    self.paint,
                );
            }
            None => (),
        }
    }

    fn start_clip(&mut self, dim: &skip::DivW) {
        self.canvas.save();
        let rect = skia_safe::Rect::from_xywh(dim.pos.x, dim.pos.y, dim.size.x, dim.size.y);
        let mut path = skia_safe::Path::rect(&rect, None);
        self.canvas.clip_path(&path, None, Some(true));
    }
    fn end_clip(&mut self) {
        self.canvas.restore();
    }

    fn mouse_pos(&mut self) -> skip::Vec2<f32> {
        (self.mouse_pos).into()
    }

    fn mouse_state(&mut self) -> &Vec<skip::On> {
        self.on
    }
}

struct WinitRenderer<T: UserEvent + 'static, A: AppController<T, Shared>, Shared> {
    windows: HashMap<winit::window::WindowId, Window<Shared, T>>,
    on: Vec<skip::On>,
    //key: Vec<skip::Key>,
    mouse_pos: skip::Vec2<f32>,
    current_focus: winit::window::WindowId,
    proxy: winit::event_loop::EventLoopProxy<T>,
    app: A,
    paint: skia_safe::Paint,
    fonts: Vec<skia_safe::Font>,
    font_mgr: skia_safe::FontMgr,
    images: Vec<skia_safe::Image>,
}

pub struct Context<'skip, Shared, T: UserEvent + 'static> {
    windows: &'skip mut HashMap<winit::window::WindowId, Window<Shared, T>>,
    event_loop: &'skip winit::event_loop::ActiveEventLoop,
    fonts: &'skip mut Vec<skia_safe::Font>,
    font_mgr: &'skip mut skia_safe::FontMgr,
    images: &'skip mut Vec<skia_safe::Image>,
}

impl<'skip, Shared, T: UserEvent + 'static> Context<'skip, Shared, T> {
    pub fn new_window(
        &mut self,
        attr: winit::window::WindowAttributes,
        draw_fn: DrawFn<Shared, T>,
        key_fn: Option<KeyFn<Shared, T>>
    ) -> winit::window::WindowId {
        let display_builder = glutin_winit::DisplayBuilder::new()
            .with_window_attributes(Some(attr.with_visible(false)))
            .with_preference(glutin_winit::ApiPreference::FallbackEgl);
        let template = glutin::config::ConfigTemplateBuilder::new().with_alpha_size(8);
        let (window, config) = display_builder
            .build(self.event_loop, template, |mut config| {
                config.next().unwrap()
            })
            .unwrap();
        let window = window.unwrap();
        let raw_handle = window.window_handle().unwrap();
        let gl_display = config.display();

        let context_attr = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::OpenGl(None)) // I just pick whatever version here, idk my laptop pretty old
            .build(Some(raw_handle.into()));
        let width = NonZeroU32::new(window.inner_size().width.max(1)).unwrap();
        let height = NonZeroU32::new(window.inner_size().height.max(1)).unwrap();
        let gl_attr =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(raw_handle.into(), width, height);

        // now this is where the fun stuff starts
        let not_current = unsafe { gl_display.create_context(&config, &context_attr).unwrap() };
        let gl_surface = unsafe { gl_display.create_window_surface(&config, &gl_attr).unwrap() };

        let context = not_current.make_current(&gl_surface).unwrap();
        // We load opengl function pointers here
        gl::load_with(|s| {
            let cstr = CString::new(s).unwrap();
            gl_display.get_proc_address(&cstr) as *const _
        });

        // basically just a bunch config for skia
        let interface = skia_safe::gpu::gl::Interface::new_native().unwrap();
        let mut gr_context =
            skia_safe::gpu::ganesh::gl::direct_contexts::make_gl(interface, None).unwrap();
        let fb_info = {
            let mut fboid: gl::types::GLint = 0;
            unsafe {
                gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid);
            }
            skia_safe::gpu::gl::FramebufferInfo {
                fboid: fboid as u32,
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                protected: skia_safe::gpu::Protected::No, // you want access to the fb info y'know
            }
        };
        let size = window.inner_size();
        let backend_render_target = skia_safe::gpu::backend_render_targets::make_gl(
            (size.width as i32, size.height as i32),
            0,
            8,
            fb_info,
        );

        // now build the damn canvas finally
        let surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
            &mut gr_context,
            &backend_render_target,
            skia_safe::gpu::SurfaceOrigin::BottomLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None,
        )
        .unwrap();
        let id = window.id();
        self.windows.insert(
            id.clone(),
            Window {
                window,
                surface,
                dr_context: gr_context,
                skia_context: context,
                fb_info,
                gl_surface,
                //focused: false,
                draw_fn,
                key_fn,
                next_redraw: None,
                redraw_policy: Redraw::FocusOnly,
            },
        );
        id
    }

    pub fn destroy(&mut self, id: &winit::window::WindowId) {
        self.windows.remove(id);
    }

    pub fn change_draw_fn(&mut self, id: &winit::window::WindowId, draw_fn: DrawFn<Shared, T>) {
        if let Some(window) = self.windows.get_mut(id) {
            window.draw_fn = draw_fn
        }
    }

    pub fn change_key_fn(&mut self, id: &winit::window::WindowId, key_fn: Option<KeyFn<Shared, T>>) {
        if let Some(window) = self.windows.get_mut(id) {
            window.key_fn = key_fn
        }
    }



    pub fn new_font(
        &mut self,
        data: &[u8],
        size: f32,
        font_id: Option<skip::Font>,
    ) -> Result<skip::Font, ()> {
        let tf = self.font_mgr.new_from_data(data, None);
        match tf {
            Some(tf) => {
                let font = skia_safe::Font::from_typeface(&tf, Some(size));
                match font_id {
                    Some(id) => {
                        self.fonts.insert(id, font);
                        Ok(id)
                    }
                    None => {
                        self.fonts.push(font);
                        Ok(self.fonts.len() - 1)
                    }
                }
            }
            None => Err(()),
        }
    }

    pub fn new_image(
        &mut self,
        data: &[u8],
        img_id: Option<skip::ImageId>,
    ) -> Result<skip::ImageId, ()> {
        let data = skia_safe::Data::new_copy(data);
        match skia_safe::Image::from_encoded(data) {
            Some(img) => match img_id {
                Some(id) => {
                    self.images.insert(id, img);
                    Ok(id)
                }
                None => {
                    self.images.push(img);
                    Ok(self.images.len() - 1)
                }
            },
            None => Err(()),
        }
    }

    pub fn set_visible(&mut self, id: &winit::window::WindowId, visible: bool) {
        if let Some(window) = self.windows.get_mut(id) {
            window.window.set_visible(visible);
        }
    }

    pub fn request_redraw(&mut self, id: &winit::window::WindowId) {
        if let Some(window) = self.windows.get_mut(id) {
            window.window.request_redraw();
        }
    }

    pub fn get_window_size(&mut self, id: &winit::window::WindowId) -> Option<skip::Vec2<f32>> {
        if let Some(window) = self.windows.get(id) {
            let size = window.window.inner_size();
            return Some((size.width as f32, size.height as f32).into());
        }
        None
    }

    pub fn exit(&mut self) {
        self.event_loop.exit();
    }
}

impl<T: UserEvent + 'static, A: AppController<T, Shared>, Shared>
    winit::application::ApplicationHandler<T> for WinitRenderer<T, A, Shared>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.app.bootstrap(Context {
            windows: &mut self.windows,
            event_loop,
            fonts: &mut self.fonts,
            font_mgr: &mut self.font_mgr,
            images: &mut self.images,
        });
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: T) {
        self.app.user_event(
            event,
            Context {
                windows: &mut self.windows,
                event_loop,
                fonts: &mut self.fonts,
                font_mgr: &mut self.font_mgr,
                images: &mut self.images,
            },
        );
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let now = Instant::now();

        let mut next: Option<Instant> = None;

        for window in self.windows.values_mut() {
            if let Some(deadline) = window.next_redraw {
                if deadline <= now {
                    window.window.request_redraw();

                    // schedule the next frame
                    window.next_redraw = Some(deadline + Duration::from_millis(16));
                }

                next = Some(match next {
                    Some(old) => old.min(window.next_redraw.unwrap()),
                    None => window.next_redraw.unwrap(),
                });
            }
        }

        match next {
            Some(deadline) => {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(deadline));
            }
            None => {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
            }
        }
        //todo!("Create the timer here!!")
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match self.windows.get_mut(&window_id) {
            None => (),
            Some(window) => match event {
                winit::event::WindowEvent::KeyboardInput { event, .. } => {
                    let key = match event.physical_key {
                        winit::keyboard::PhysicalKey::Code(c) => {
                            let key = keycode_to_str(c);
                            let skip_key = match event.state {
                                winit::event::ElementState::Pressed => skip::Key::Press(key),
                                winit::event::ElementState::Released => skip::Key::Release(key),
                            };
                            skip_key
                        }
                        winit::keyboard::PhysicalKey::Unidentified(_) => {
                            skip::Key::Press("Unknown")
                        }
                    };
                    if let Some(key_fn) = &window.key_fn {
                        (key_fn.0)(self.app.share_resource(), key, &self.proxy);
                    }
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    let button = match button {
                        winit::event::MouseButton::Left => skip::Mouse::Left,
                        winit::event::MouseButton::Right => skip::Mouse::Right,
                        winit::event::MouseButton::Middle => skip::Mouse::Middle,
                        _ => skip::Mouse::Unknown,
                    };
                    let state = match state {
                        winit::event::ElementState::Pressed => skip::State::Pressed,
                        winit::event::ElementState::Released => skip::State::Released,
                    };
                    self.on.push((button, state));
                }
                winit::event::WindowEvent::RedrawRequested => {
                    let redraw = match window.redraw_policy {
                        Redraw::FocusOnly => {
                            if self.current_focus == window_id {
                                true
                            } else {
                                false
                            }
                        }
                        Redraw::Always => true,
                    };
                    if !redraw {
                        return;
                    }
                    let canvas = window.surface.canvas();
                    let window_dim = window.window.inner_size();
                    canvas.clear(skia_safe::Color::WHITE); 
                    let duration = (window.draw_fn.0)(
                        self.app.share_resource(),
                        skip::Horizontal::new(Canvas {
                            on: &self.on,
                            mouse_pos: &self.mouse_pos,
                            //key: &self.key,
                            canvas,
                            paint: &mut self.paint,
                            fonts: &self.fonts,
                            images: &self.images,
                            window_dim: (window_dim.width as f32, window_dim.height as f32).into(),
                        }),
                        &self.proxy,
                    );
                    window.dr_context.flush_and_submit();
                    //window.dr_context.flush_and_submit_surface(&mut window.surface, None);
                    window
                        .gl_surface
                        .swap_buffers(&window.skia_context)
                        .unwrap();
                    self.on.clear();
//                    self.key.clear();
                    if let Some(d) = duration {
                        window.next_redraw = Some(Instant::now() + d);
                    }
                }
                winit::event::WindowEvent::Resized(size) => {
                    let backend_render_target = skia_safe::gpu::backend_render_targets::make_gl(
                        (size.width as i32, size.height as i32),
                        0,
                        8,
                        window.fb_info,
                    );
                    window.surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
                        &mut window.dr_context,
                        &backend_render_target,
                        skia_safe::gpu::SurfaceOrigin::BottomLeft,
                        skia_safe::ColorType::RGBA8888,
                        None,
                        None,
                    )
                    .unwrap();
        
                }
                winit::event::WindowEvent::CloseRequested => {
                    self.windows.remove(&window_id);
                }
                winit::event::WindowEvent::Focused(_) => {
                    self.current_focus = window_id;
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    let logical = position.to_logical::<f32>(window.window.scale_factor());
                    self.mouse_pos = (logical.x, logical.y as f32).into();
                    //                    dbg!(logical);
                }
                _ => (),
            },
        }
    }
}

fn keycode_to_str(key: winit::keyboard::KeyCode) -> &'static str {
    use winit::keyboard::*;
    match key {
        // Letters
        KeyCode::KeyA => "a",
        KeyCode::KeyB => "b",
        KeyCode::KeyC => "c",
        KeyCode::KeyD => "d",
        KeyCode::KeyE => "e",
        KeyCode::KeyF => "f",
        KeyCode::KeyG => "g",
        KeyCode::KeyH => "h",
        KeyCode::KeyI => "i",
        KeyCode::KeyJ => "j",
        KeyCode::KeyK => "k",
        KeyCode::KeyL => "l",
        KeyCode::KeyM => "m",
        KeyCode::KeyN => "n",
        KeyCode::KeyO => "o",
        KeyCode::KeyP => "p",
        KeyCode::KeyQ => "q",
        KeyCode::KeyR => "r",
        KeyCode::KeyS => "s",
        KeyCode::KeyT => "t",
        KeyCode::KeyU => "u",
        KeyCode::KeyV => "v",
        KeyCode::KeyW => "w",
        KeyCode::KeyX => "x",
        KeyCode::KeyY => "y",
        KeyCode::KeyZ => "z",

        // Digits
        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",

        // Symbols
        KeyCode::Backquote => "`",
        KeyCode::Backslash => "\\",
        KeyCode::BracketLeft => "[",
        KeyCode::BracketRight => "]",
        KeyCode::Comma => ",",
        KeyCode::Equal => "=",
        KeyCode::Minus => "-",
        KeyCode::Period => ".",
        KeyCode::Quote => "'",
        KeyCode::Semicolon => ";",
        KeyCode::Slash => "/",
        KeyCode::Space => " ",

        // Everything else keeps its variant name
        KeyCode::AltLeft => "AltLeft",
        KeyCode::AltRight => "AltRight",
        KeyCode::Backspace => "Backspace",
        KeyCode::CapsLock => "CapsLock",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Escape",
        KeyCode::Tab => "Tab",
        KeyCode::ArrowUp => "ArrowUp",
        KeyCode::ArrowDown => "ArrowDown",
        KeyCode::ArrowLeft => "ArrowLeft",
        KeyCode::ArrowRight => "ArrowRight",
        // ...continue for the remaining variants...
        _ => "Unknown",
    }
}
