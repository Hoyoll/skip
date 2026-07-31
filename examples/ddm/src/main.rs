use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread::{self, JoinHandle}, time::Duration,
};

use reqwest::blocking::Client;
use serde_json::Value;
use skip::{Clip, Div, Font, Leak, Mouse, Mouses, Plain, Proc, Set, State, Text, Vertical, Wrap};
use skip_skia::{AppController, Canvas, Event, run_app};
use winit::{
    event_loop::EventLoopProxy,
    window::{WindowAttributes, WindowId},
};

mod asset {
    pub const ROBOTO: &[u8] = include_bytes!("../assets/fonts/roboto.ttf");
    pub const FIRA_CODE: &[u8] = include_bytes!("../assets/fonts/fira_code.ttf");
}

mod color {
    pub const BG: (u8, u8, u8,u8) = (16, 20, 28, 255);
}

use serde::{Deserialize, Serialize};
#[derive(Debug,Clone, Serialize, Deserialize)]
enum EntryType {
    #[serde(rename = "dir")]
    Directory,

    #[serde(rename = "file")]
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    name: String,
    url: String,
    //sha: String,
    download_url: Option<String>,
    #[serde(rename = "type")]
    kind: EntryType,
}

struct EntryList {
    list: Option<Vec<Entry>>,
    sender: Sender<String>,
}

struct App {
    //id: WindowId,
    //player: Player,
    fonts: Fonts,
    entry_list: EntryList,
    proxy: Option<EventLoopProxy<Music>>,
    temp: Option<Receiver<String>>,
}

enum Music {
    NewEntry(Vec<Entry>),
    Play(usize, Vec<Entry>),
    Error(String),
}

struct Fonts {
    roboto: Font,
    fira_code: Font,
}

struct Player {}

impl App {
    fn new() -> Self {
        let (s, r) = channel::<String>();
        Self {
            entry_list: EntryList {
                list: None,
                sender: s,
            },
            fonts: Fonts { roboto: 0, fira_code: 1 },
            proxy: None,
            temp: Some(r),
        }
    }

    fn build_client(&mut self, client: Client) { 
        let r = self.temp.take().unwrap(); 
        let proxy = self.proxy.clone().unwrap();
        let handle = thread::spawn(move || {
            for url in r.iter() {
                //println!("{}", url);
                if let Ok(response) = client.get(url).send() {
                    response.text().map(|json| {
                        serde_json::from_str(&json).map(|entry: Vec<Entry>| {
                            //println!("fresh entry!");
                            proxy.send_event(Music::NewEntry(entry));
                        });
                    });
                }
            }
            //println!("end!");
        });
    }
}

impl AppController<Music> for App {
    fn on_draw(
        &mut self,
        on_window: winit::window::WindowId,
        mut layout: skip::Horizontal<skip_skia::Canvas>,
    ) -> Option<std::time::Duration> {
        let canvas_size = layout.canvas_size();
        layout.add(|background:Div<_>| {
            background
            .size::<Set>(&canvas_size)
            .render::<Plain<_>>(color::BG)
            .child::<Div<_>, Leak>(|list| {
                if let Some(proxy) = &self.proxy {
                    list.proc((&mut self.entry_list, (proxy, self.fonts.fira_code)))
                } else {
                    list
                }
            })
        });
        Some(Duration::from_millis(16))
    }

    fn bootstrap<'skip>(
        &mut self,
        mut context: skip_skia::Context<'skip>,
        event: EventLoopProxy<Music>,
    ) {
        self.proxy = Some(event);

        let client = reqwest::blocking::Client::builder()
            .user_agent("ddm/0.1.0")
            .build();
        match client {
            Ok(c) => {
                self.build_client(c);
                self.entry_list.sender.send("https://api.github.com/repos/Hoyoll/musics/contents".into());
            }
            Err(_) => {
                context.exit();
            }
        }

        self.fonts.roboto = context.new_font(&asset::ROBOTO, Some(0)).unwrap();
        self.fonts.fira_code = context.new_font(&asset::FIRA_CODE, Some(1)).unwrap();

        let attr = WindowAttributes::default()
            .with_resizable(false)
            .with_title("DDM!");
        let id = context.new_window(attr);
        context.request_redraw(&id);
        context.set_visible(&id, true);
    }

    fn on_user_event<'skip>(&mut self, user_event: Music, context: skip_skia::Context<'skip>) {
        match user_event {
            Music::NewEntry(entry) => {
                println!("new entry!");
                self.entry_list.new_entry(entry);
            }
            _ => (),
        }
    }
}
fn main() {
    run_app(App::new());
    //println!("{}",response.text().unwrap());
}

impl EntryList {
    fn new_entry(&mut self, list: Vec<Entry>) {
        self.list = Some(list);
    }
}

impl<'skip> Proc<'skip, Canvas<'skip>> for &mut EntryList {
    type Widget = Div<Canvas<'skip>>;
    type Arg = (&'skip EventLoopProxy<Music>, Font);

    fn consume(self, widget: Self::Widget, (proxy, font): Self::Arg) -> Self::Widget {
        widget.child::<Vertical<_>, Clip>(|vertical| match &self.list {
            None => vertical, //idk, currently just zonk XD
            Some(list) => vertical
                .gap(5.0)
                .iter(list.iter().enumerate(), |text: Text<Wrap,_>, (idx, entry)| {
                text
                .font_id(font)
                .size(40.0)
                .text(&entry.name)
                .expr((&entry.download_url, |text, kind| {
                    match kind {
                        None => {
                            text
                            .on::<Mouses>(|mouse| {
                                match mouse {
                                    (Mouse::Left, State::Released) => {
                                        println!("released!");
                                        self.sender.send(entry.url.clone());
                                    }
                                    _ => ()
                                }
                            })
                            .render((255, 255, 255, 255))
                        }
                        Some(_) => {
                            text
                            .on::<Mouses>(|mouse| {
                                match mouse {
                                    (Mouse::Left, State::Released) => {
                                        proxy.send_event(Music::Play(idx - 1, list.clone()));
                                    }
                                    _ => ()
                                }
                            })
                            .render((255, 0, 255, 255))
                        }
                    }
                }))
            }),
        })
    }
}
