use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::services::{IntervalService, ConsoleService, Task};

use std::time::Duration;

struct Model {
    interval: IntervalService,
    console: ConsoleService,
}

enum Msg {
    AnimTick(f32)
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut interval = IntervalService::new();
        interval.spawn(Duration::from_millis(50), link.send_back(|_| Msg::AnimTick(0.05)));
        let mut console = ConsoleService::new();
        console.log("Starting up");
        Model {
            interval: interval,
            console: console,
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AnimTick(x) => {
                self.console.log("anim tick");
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <svg viewBox="0 0 500 500", xmlns="http://www.w3.org/2000/svg",>
                    <rect x=10, y=10, width=50, height=50,></rect>
                </svg>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
