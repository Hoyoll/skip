use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread::{self, JoinHandle},
};

use serde_json::Value;
use skip::{Clip, Div, Font, Leak, Mouse, Mouses, Proc, State, Text, Vertical};
use skip_skia::{AppController, Canvas, Event};
use winit::{
    event_loop::EventLoopProxy,
    window::{WindowAttributes, WindowId},
};

use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
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
    //#[serde(rename = "type")]
    //kind: EntryType,
}

struct EntryList {
    list: Option<Vec<Entry>>,
    sender: Sender<String>,
}

struct App {
    //id: WindowId,
    //player: Player,
    entry_list: EntryList,
    proxy: Option<EventLoopProxy<Music>>,
    temp: Option<Receiver<String>>,
}

enum Music {
    NewEntry(Vec<Entry>),
    Play(usize, Vec<Entry>),
    Error(String),
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
            proxy: None,
            temp: Some(r),
        }
    }
}

impl AppController<Music> for App {
    fn on_draw(
        &mut self,
        on_window: winit::window::WindowId,
        layout: skip::Horizontal<skip_skia::Canvas>,
    ) -> Option<std::time::Duration> {
        None
    }

    fn bootstrap<'skip>(
        &mut self,
        mut context: skip_skia::Context<'skip>,
        event: EventLoopProxy<Music>,
    ) {
        let client = reqwest::blocking::Client::builder()
            .user_agent("ddm/0.1.0")
            .build();
        match client {
            Ok(c) => {
                //let (s, r) = channel::<String>();
                let r = self.temp.take().unwrap();
                let ev = event.clone();
                //self.entry_list.sender = s;
                thread::spawn(move || {
                    for url in r.recv().iter() {
                        if let Ok(response) = c.get(url).send() {
                            response.text().map(|json| {
                                serde_json::from_str(&json).map(|entry: Vec<Entry>| {
                                    ev.send_event(Music::NewEntry(entry));
                                });
                            });
                        }
                    }
                });
            }
            Err(_) => {
                context.exit();
            }
        }

        let attr = WindowAttributes::default()
            .with_resizable(false)
            .with_title("DDM!");
        context.new_window(attr);
    }

    fn on_user_event<'skip>(&mut self, user_event: Music, context: skip_skia::Context<'skip>) {
        match user_event {
            Music::NewEntry(entry) => {
                self.entry_list.new_entry(entry);
            }
            _ => (),
        }
    }
}
fn main() {
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
            None => vertical ,//idk, currently just zonk XD
            Some(list) => vertical
                .iter(list.iter().enumerate(), |text: Text<_>, (idx, entry)| {
                    text
                    .font_id(font)
                    .size(40.0)
                    .text(&entry.name)
                    .on::<Mouses>(|mouse|{
                        match mouse {
                            (Mouse::Left, State::Released) => {
                                proxy.send_event(Music::Play(idx, list.clone()));
                            }
                            _ => ()
                        }
                    })
                    .render((255, 255, 255, 255))
                }),
        })
    }
}
