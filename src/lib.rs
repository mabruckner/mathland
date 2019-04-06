#![recursion_limit="128"]

use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::services::{IntervalService, ConsoleService, Task};
use yew::events::{KeyPressEvent, KeyDownEvent, KeyUpEvent};

use std::time::Duration;

use stdweb::web::{window, IEventTarget};
use stdweb::web::event::IKeyboardEvent;

use rand::distributions::{Normal, Distribution};
use rand::prelude::*;

mod problem;
use problem::*;

mod textbox;
use textbox::*;

mod enemy;
use enemy::*;

mod context;
use context::*;

enum PType {
    Spark
}

pub struct Particle {
    p_type: PType,
    vel: [f32; 2],
    pos: [f32; 2],
    grav: [f32; 2],
    life: f32
}

impl Particle {
    fn tick(&mut self, delta: f32) {
        for i in 0..2 {
            self.vel[i] += self.grav[i] * delta;
            self.pos[i] += self.vel[i] * delta;
        }
        self.life -= delta;
    }
}

pub struct Model {
    pub interval: IntervalService,
    pub console: ConsoleService,
    pub dir: Direction,
    pub text: TextBox,
    pub problem: Option<Box<Problem>>,
    pub enemy: Box<Enemy>,
    pub enemy_props: EnemyProps,
    pub ctx: Context,
    pub particles: Vec<Particle>,
    pub state: FighterState,
    pub _anim_task: Box<Task>,
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

impl Model {
    fn blast(&mut self, pos: [f32; 2], vel: [f32; 2], spread: f32, count: usize, life: f32) {
        let mut rng = SmallRng::from_entropy();
        let xdst = Normal::new(vel[0] as f64, spread as f64);
        let ydst = Normal::new(vel[1] as f64, spread as f64);
        for i in 0..count {
            self.particles.push(Particle {
                p_type: PType::Spark,
                pos: pos,
                vel: [xdst.sample(&mut rng) as f32, ydst.sample(&mut rng) as f32],
                life: life,
                grav: [0.0, 400.0]
            });
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut interval = IntervalService::new();
        let handle = interval.spawn(Duration::from_millis(50), link.send_back(|_| Msg::AnimTick(0.05)));
        let mut console = ConsoleService::new();
        console.log("Starting up");
        let callback = link.send_back(|e:KeyDownEvent| Msg::KeyDown(e));
        window().add_event_listener(move |e: KeyDownEvent| callback.emit(e));
        let e = Box::new(Orb::new());
        let props = e.get_properties();
        Model {
            interval: interval,
            console: console,
            dir: Direction::new(),
            text: TextBox::new(),
            problem: Some(Box::new(TextProblem {
                problem: "1 + 1".into(),
                answer: "2".into()
            })),
            enemy: e,
            enemy_props: props,
            ctx: Context {
                anim_t: 0.0
            },
            particles: Vec::new(),
            state: FighterState {
                health: 1.0,
            },
            _anim_task: Box::new(handle),
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AnimTick(x) => {
                self.ctx.anim_t += x;
                for mut particle in self.particles.iter_mut() {
                    particle.tick(x);
                }
                for i in (0..self.particles.len()).rev() {
                    if self.particles[i].life < 0.0 {
                        self.particles.swap_remove(i);
                    }
                }
                true
            },
            Msg::KeyDown(x) => {
                if x.key() == "Enter" && self.problem.is_some() {
                    let correct = if let Some(ref p) = self.problem {
                        p.test_correct(&self.text.text)
                    } else {
                        false
                    };
                    if correct {
                        self.enemy.damage(1.0);
                        self.blast([820.0, 250.0], [200.0, 200.0], 500.0, 10, 4.0);
                    } else {
                        self.state.health -= 0.2;
                        self.blast([180.0, 250.0], [-200.0, 200.0], 500.0, 10, 4.0);
                        self.console.log("INCORRECT");
                        // incorrect
                    }
                    self.problem = Some(self.enemy.generate_problem());
                    self.text = TextBox::new();
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
        let enemy_state = self.enemy.get_state(); 
        html! {
            <div onkeydown=|_| Msg::AnimTick(1.0),>
                <svg onkeypress=|_| Msg::AnimTick(1.0), viewBox="0 0 1000 600", xmlns="http://www.w3.org/2000/svg",>
                    <g transform="translate(875, 400)",>{ self.enemy_props.card.render(&self.ctx) }</g>
                    <g class="particles",>
                    {for self.particles.iter().map(|particle| {
                            html!{
                                <g class="particle",>
                                <circle cx={particle.pos[0]}, cy={particle.pos[1]}, r=5,></circle>
                                <line x1={particle.pos[0]},
                                y1={particle.pos[1]},
                                x2={particle.pos[0]-particle.vel[0]*0.05},
                                y2={particle.pos[1]-particle.vel[1]*0.05},></line>
                                </g>
                            }
                            
                        })
                    }
                    </g>
                    <g transform="translate(750, 430)",>{ stats_card(&self.enemy_props) }</g>
                    <rect class="problem_card", x=250, y=20, width=500, height=500, rx=10, ry=10,></rect>
                    <rect class="text_box", x=250, y=530, width=500, height=50, rx=10, ry=10,></rect>
                    <g transform="translate(500, 300) scale(5)",>
                        {
                            if let Some(ref p) = self.problem {
                                p.render()
                            } else {
                                html! { <text>{"no problem"}</text> }
                            }
                        }
                    </g>
                    <text x=260, y=567, class="answertext",>{&self.text.text}</text>
                    <g transform="translate(0, 20)",>
                        { health_bar(self.state.health, 475.0) }
                    </g>
                    <g transform="scale(-1.0, 1.0) translate(-1000, 20)",>
                        { health_bar(enemy_state.health, 475.0) }
                    </g>
                </svg>
            </div>
        }
    }
}

fn stats_card(props: &EnemyProps) -> Html<Model> {
    html! {
        <g class="stats",>
            <path d="M 0 0 L 200 0 L 185 70 L 0 70",></path>
            <text x=20, y=50, class="level",>{ format!("Level {}", &props.level) }</text>
            <text x=20, y=65, class="class",>{ &props.class }</text>
            <text x=20, y=30, class="name",>{ &props.name }</text>
        </g>
    }
}

fn health_bar(health: f64, width: f64) -> Html<Model> {
    let start = 100.0 - 20.0;
    let end = width;
    let position = end*health + start*(1.0-health);
    html! {
        <g class="health_bar",>
            <path class="bar", d={format!("M {} 20 L {} 20 L {} 30 L {} 30", start, position, position-5.0, start-5.0) },></path>
            <path d="M0 0 L100 0 L50 100 L 0 100",></path>
        </g>
    }
}
