use std::{collections::HashMap, ffi::CString, num::NonZeroU32};

use glutin::{context::NotCurrentGlContext, display::{GetGlDisplay, GlDisplay}};
use raw_window_handle::HasWindowHandle;

pub enum Control {
    Redraw,
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
        ui: skip::Div<&mut Window>, 
        proxy: &winit::event_loop::EventLoopProxy<T>
    ) -> Control;
}

struct Window {
    pub on: Vec<skip::On>,
    pub window_dim: skip::Vec2<f32>,
    pub mouse_pos: skip::Vec2<f32>,
    pub key: Vec<skip::Key>,
    pub window: winit::window::Window,
}

impl skip::Renderer for &mut Window {
    fn render_text<'skip>(&mut self, text: &skip::TextW<'skip>) {
        
    }
    fn render_div(&mut self, div: &skip::DivW) {
        
    }
    fn on_text<'skip, F: FnMut(&mut skip::TextW<'skip>, &skip::On)>(&mut self,text: &mut skip::TextW<'skip>, mut f: F) {
        for on in &self.on {
           f(text, on); 
       }
    }
    fn on_div<F: FnMut(&mut skip::DivW, &skip::On)>(&mut self,div: &mut skip::DivW, mut f: F) {
        let limit_x = div.pos.x + div.dim.x;
        let limit_y = div.pos.y + div.dim.y;
        if (self.mouse_pos.x >= div.pos.x && self.mouse_pos.x >= limit_x) && (self.mouse_pos.y >= div.pos.y && self.mouse_pos.y >= limit_y)  {
            f(div, &skip::On::Hover(skip::Vec2::new(self.mouse_pos.x, self.mouse_pos.y)));
            for on in &self.on {
                f(div, on);
            }
        } 
    }
    fn key_div<F: FnMut(&mut skip::DivW, &skip::Key)>(&mut self,div: &mut skip::DivW, mut f: F) {
       for key in &self.key {
            f(div, key);
        } 
    }
    fn key_text<'skip,F: FnMut(&mut skip::TextW<'skip>, &skip::Key)>(&mut self,text: &mut skip::TextW<'skip>, mut f: F) {
       for key in &self.key {
           f(text, key) 
        } 
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
        //let window = self.event_loop.create_window(attr).unwrap();
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
        todo!("HERE!"); 
        window.id()
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
                //keycode_to_str(event.physical_key);
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
                    match self.app.draw(window_id, skip::Div::new((),window), &self.proxy) {
                        Control::Kill => {
                           self.windows.remove(&window_id); 
                        }
                        Control::Redraw => {
                            window.window.request_redraw();                            
                        }
                    }
                }
                _=> ()
                }
            }
        }
    }
}

pub fn keycode_to_str(key: winit::keyboard::KeyCode) -> &'static str {
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


struct App;

enum Cool {}

impl UserEvent for Cool {
    
}

struct Proc;

impl Proc {
    fn tex(&mut self) {

    }
}

impl<'skip> skip::Proc<
    'skip,
    skip::Div<&'skip mut Window>,
    skip::Text<'skip, &'skip mut Window>,
    &'skip mut Window> for &mut Proc{
    
    fn consume(
        &mut self,
        widget: skip::Div<&'skip mut Window>,
    ) -> skip::Text<'skip, &'skip mut Window> {
        self.tex();
        widget.to_text("Hello")
    }
}

impl<'skip> skip::Proc<'skip,skip::Div<&'skip mut Window>,skip::Div<&'skip mut Window>, &'skip mut Window> for Proc {
    fn consume(&mut self, widget: skip::Div<&'skip mut Window>) -> skip::Div<&'skip mut Window> {
       widget 
    }
}

impl AppController<Cool> for App {
    fn bootstrap<'skip>(&mut self, context: Context<'skip>) {
        
    }
    fn user_event<'skip>(
        &mut self, 
        user_event: Cool, 
        context: Context<'skip>
    ) {


        
    }

    fn draw(
        &mut self, 
        on: winit::window::WindowId, 
        ui: skip::Div<&mut Window>, 
        proxy: &winit::event_loop::EventLoopProxy<Cool>
    ) -> Control {
        ui
            .pos((10.0, 10.0))
            .color((10, 255, 33,12))
            .dim((100.0,100.0))
            .on(|div, on| {
                match on {
                    skip::On::Hover(_) => {
                        div.color.g = 255;
                    }
                    skip::On::Press(skip::Mouse::Left) => {
                        div.color.a = 0;
                    }
                    _ => ()
                }
                
            })
            .render()
                .children(|c| {
                    c.text(|t| {
                        t
                    })
                })
                .to_text(("Hello!", 0, 1))
                .on(|text, on| {
                    match on {
                        skip::On::Hover(_) => {
                            text.color.a = 255;
                        }
                        skip::On::Press(skip::Mouse::Left) => {
                            text.color.b = 0;
                        }
                        _ => ()
                }
                })
                .render();
        Control::Redraw 
    }
}

fn main() {
    println!("Hello, world!");
}
