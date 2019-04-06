use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::services::{IntervalService, ConsoleService, Task};
use yew::events::{KeyPressEvent, KeyDownEvent, KeyUpEvent};

use std::time::Duration;

use stdweb::web::{window, IEventTarget};
use stdweb::web::event::IKeyboardEvent;

mod problem;
use problem::*;

mod textbox;
use textbox::*;

pub struct Model {
    pub interval: IntervalService,
    pub console: ConsoleService,
    pub dir: Direction,
    pub text: TextBox,
    pub problem: Option<Box<Problem>>
}

pub struct Direction {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Direction {
    fn new() -> Self {
        Direction {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}


pub enum Msg {
    AnimTick(f32),
    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut interval = IntervalService::new();
        interval.spawn(Duration::from_millis(50), link.send_back(|_| Msg::AnimTick(0.05)));
        let mut console = ConsoleService::new();
        console.log("Starting up");
        let callback = link.send_back(|e:KeyDownEvent| Msg::KeyDown(e));
        window().add_event_listener(move |e: KeyDownEvent| callback.emit(e));
        Model {
            interval: interval,
            console: console,
            dir: Direction::new(),
            text: TextBox::new(),
            problem: Some(Box::new(TextProblem {
                problem: "1 + 1".into(),
                answer: "2".into()
            }))
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AnimTick(x) => {
                self.console.log("anim tick");
                true
            },
            Msg::KeyDown(x) => {
                if x.key() == "Enter" && self.problem.is_some() {
                } else {
                    self.text.down(&x);
                    self.console.log(&format!("{:?}", x.key()));
                }
                true
            },
            Msg::KeyUp(x) => {
                self.text.up(&x);
                self.console.log(&format!("{:?}", x));
                true
            },
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div onkeydown=|_| Msg::AnimTick(1.0),>
                <svg onkeypress=|_| Msg::AnimTick(1.0), viewBox="0 0 800 600", xmlns="http://www.w3.org/2000/svg",>
                    <rect class="problem_card", x=150, y=20, width=500, height=500, rx=10, ry=10,></rect>
                    <rect class="text_box", x=150, y=530, width=500, height=50, rx=10, ry=10,></rect>
                    <g transform="translate(400, 300) scale(5)",>
                        {
                            if let Some(ref p) = self.problem {
                                p.render()
                            } else {
                                html! { <text>{"no problem"}</text> }
                            }
                        }
                    </g>
                    <text x=150, y=530,>{&self.text.text}</text>
                </svg>
            </div>
        }
    }
}
