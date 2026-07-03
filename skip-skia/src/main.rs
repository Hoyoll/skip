use std::{collections::HashMap, ffi::CString, num::NonZeroU32};

use glutin::{context::NotCurrentGlContext, display::{GetGlDisplay, GlDisplay}, surface::GlSurface};
use raw_window_handle::HasWindowHandle;

pub enum Control {
    Redraw,
    Suspend,
    Kill,
}

pub trait UserEvent {}
pub trait AppController<T: UserEvent> {
    fn bootstrap<'skip>(&mut self, context: Context<'skip>);
    fn user_event<'skip>(
        &mut self, 
        user_event: T, 
        context: Context<'skip>
    );
    fn draw(
        &mut self, 
        on: winit::window::WindowId, 
        ui: skip::Div<&mut Canvas>, 
        proxy: &winit::event_loop::EventLoopProxy<T>
    ) -> Control;
}

struct Window {
    on: Vec<skip::On>,
    mouse_pos: skip::Vec2<f32>,
    key: Vec<skip::Key>,
    window: winit::window::Window,
    surface: skia_safe::Surface,
    dr_context: skia_safe::gpu::DirectContext,
    skia_context: glutin::context::PossiblyCurrentContext,
    fb_info: skia_safe::gpu::gl::FramebufferInfo,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

struct Canvas<'skip> {
    on: &'skip Vec<skip::On>,
    mouse_pos: &'skip skip::Vec2<f32>,
    key: &'skip Vec<skip::Key>,
    canvas: &'skip skia_safe::Canvas
}

impl<'a> skip::Renderer for &mut Canvas<'a> {
    fn render_div(&mut self, div: &skip::DivW) {
        
    }
    fn text_size<'skip>(&mut self, text: &skip::TextW<'skip>) -> skip::Vec2<f32> {
        ().into()
    }
    fn render_text<'skip>(&mut self, text: &skip::TextW<'skip>) {
        
    }
    fn on_text<'skip, F: FnMut(&mut skip::TextW<'skip>, &skip::On)>(&mut self,text: &mut skip::TextW<'skip>, f: F) {
        
    }
    fn on_div<F: FnMut(&mut skip::DivW, &skip::On)>(&mut self,div: &mut skip::DivW, f: F) {
        
    }
    fn key_div<F: FnMut(&mut skip::DivW, &skip::Key)>(&mut self,div: &mut skip::DivW, f: F) {
        
    }
    fn key_text<'skip, F: FnMut(&mut skip::TextW<'skip>, &skip::Key)>(&mut self,text: &mut skip::TextW<'skip>, f: F) {
        
    }
    fn start_clip(&mut self, dim: &skip::DivW) {
        
    }
    fn end_clip(&mut self) {
        
    }
}

struct WinitRenderer<T: UserEvent + 'static, A: AppController<T>> {
    windows: HashMap<winit::window::WindowId, Window>,
    proxy: winit::event_loop::EventLoopProxy<T>,
    app: A,
}

pub struct Context<'skip> {
    windows: &'skip mut HashMap<winit::window::WindowId, Window>,
    event_loop: &'skip winit::event_loop::ActiveEventLoop,
}


impl<'skip> Context<'skip> {
    pub fn new_window(&mut self, attr: winit::window::WindowAttributes) -> winit::window::WindowId { 
        let display_builder = glutin_winit::DisplayBuilder::new()
            .with_window_attributes(Some(attr.with_visible(false)))
            .with_preference(glutin_winit::ApiPreference::FallbackEgl);
        let template = glutin::config::ConfigTemplateBuilder::new().with_alpha_size(8);
        let (window, config) = display_builder
            .build(self.event_loop, template, |mut config| config.next().unwrap())
            .unwrap();
        let window = window.unwrap();
        let raw_handle = window.window_handle().unwrap();
        let gl_display = config.display();
        
        let context_attr = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::OpenGl(None)) // I just pick whatever version here, idk my laptop pretty old
            .build(Some(raw_handle.into()));
        let width = NonZeroU32::new(window.inner_size().width.max(1)).unwrap();
        let height = NonZeroU32::new(window.inner_size().height.max(1)).unwrap();
        let gl_attr = glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
            raw_handle.into(),
            width,
            height,
        );

        // now this is where the fun stuff starts
        let not_current = unsafe {
            gl_display
                .create_context(&config, &context_attr)
                .unwrap()
        };
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&config, &gl_attr)
                .unwrap()
        };

        let context = not_current.make_current(&gl_surface).unwrap();
        // We load opengl function pointers here
        gl::load_with(|s| {
            let cstr = CString::new(s).unwrap();
            gl_display.get_proc_address(&cstr) as *const _
        });

        // basically just a bunch config for skia
        let interface = skia_safe::gpu::gl::Interface::new_native().unwrap();
        let mut gr_context = skia_safe::gpu::ganesh::gl::direct_contexts::make_gl(interface, None).unwrap();
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
        let backend_render_target =
            skia_safe::gpu::backend_render_targets::make_gl((size.width as i32, size.height as i32), 0, 8, fb_info);

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
        self.windows.insert(id.clone(), Window { 
            on: vec![], 
            mouse_pos: ().into(), 
            key: vec![], window, surface, 
            dr_context: gr_context, skia_context: context , fb_info, gl_surface 
        });
        id
    }

    pub fn destroy(&mut self, id: &winit::window::WindowId) {
        self.windows.remove(id);
    }

    pub fn set_visible(&mut self, id: &winit::window::WindowId, visible: bool) {
        if let Some(window) = self.windows.get_mut(id) {
            window.window.set_visible(visible);
        }
    }

    pub fn request_redraw(&mut self,id: &winit::window::WindowId) {
        if let Some(window) = self.windows.get_mut(id) {
            window.window.request_redraw();
        }
    }

    pub fn exit(&mut self) {
        self.event_loop.exit();
    }
}

impl<T: UserEvent + 'static, A: AppController<T>> winit::application::ApplicationHandler<T> for WinitRenderer<T, A> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.app.bootstrap(Context { windows: &mut self.windows, event_loop });        
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: T) {
        self.app.user_event(event, Context { windows: &mut self.windows, event_loop });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    )
    {
        match self.windows.get_mut(&window_id) {
            None => (),
            Some(window) => {
                match event {
                    winit::event::WindowEvent::KeyboardInput { event, .. } => {
                        match event.physical_key {
                            winit::keyboard::PhysicalKey::Code(c) => {
                                let key = keycode_to_str(c);
                                let skip_key = match event.state {
                                    winit::event::ElementState::Pressed => {
                                        skip::Key::Press(key)
                                    }
                                    winit::event::ElementState::Released => {
                                        skip::Key::Release(key)
                                    }
                                };
                                window.key.push(skip_key);
                            }
                            winit::keyboard::PhysicalKey::Unidentified(_) => {
                                window.key.push(skip::Key::Press("Unknown"));
                        }
                    }
                    }
                winit::event::WindowEvent::MouseInput { state, button,.. } => {
                    let button = match button {
                        winit::event::MouseButton::Left => {
                            skip::Mouse::Left
                        }
                        winit::event::MouseButton::Right => {
                            skip::Mouse::Right
                        }
                        winit::event::MouseButton::Middle => {
                            skip::Mouse::Middle
                        }
                        _ => skip::Mouse::Unknown
                    };
                    let state = match state {
                        winit::event::ElementState::Pressed => skip::On::Press(button),
                        winit::event::ElementState::Released => skip::On::Release(button) 
                    };
                    window.on.push(state);
                }
                winit::event::WindowEvent::RedrawRequested => {
                    let canvas = window.surface.canvas();
                    let run = self.app.draw(
                        window_id, 
                        skip::Div::new((), 
                            &mut Canvas { 
                                on: &window.on, 
                                mouse_pos: &window.mouse_pos, 
                                key: &window.key, canvas } 
                            ), 
                            &self.proxy
                        );
                    window.dr_context.flush_and_submit();
                    window.gl_surface.swap_buffers(&window.skia_context).unwrap();
                    match run {
                            Control::Kill => {
                                self.windows.remove(&window_id); 
                            }
                            Control::Redraw => {
                                window.window.request_redraw();                            
                            }
                            Control::Suspend => ()
                    }
                    
                }
                winit::event::WindowEvent::Resized(size) => {
                    let backend_render_target =
                    skia_safe::gpu::backend_render_targets::make_gl((size.width as i32, size.height as i32), 0, 8, window.fb_info);
                    window.surface = skia_safe::gpu::surfaces::wrap_backend_render_target(
                        &mut window.dr_context,
                        &backend_render_target,
                        skia_safe::gpu::SurfaceOrigin::BottomLeft,
                        skia_safe::ColorType::RGBA8888,
                        None,
                        None,
                    ).unwrap();                   
                }
                _=> ()
                }
            }
        }
    }
}

struct App;

enum Cool {}

impl UserEvent for Cool {
    
}

struct Proc;

impl Proc {
    fn tex(&mut self) {

    }
}

fn main() {
    println!("Hello, world!");
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


