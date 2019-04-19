#![recursion_limit="256"]

use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::services::{IntervalService, ConsoleService, Task};
use yew::events::{KeyDownEvent, KeyUpEvent, IKeyboardEvent};

use std::time::Duration;

use stdweb::web::{window, IEventTarget};

use rand::distributions::{Normal, Distribution};
use rand::prelude::*;

use cgmath::Vector2;

mod problem;
use problem::*;

mod textbox;
use textbox::*;

mod enemy;
use enemy::*;

mod context;
use context::*;

mod direction;
use direction::*;

pub struct Particle {
    vel: [f32; 2],
    pos: [f32; 2],
    grav: [f32; 2],
    damp: f32,
    life: f32,
}

impl Particle {
    fn tick(&mut self, delta: f32) {
        for i in 0..2 {
            self.vel[i] += self.grav[i] * delta;
            self.vel[i] *= 1.0/(self.damp*delta).exp2();
            self.pos[i] += self.vel[i] * delta;
        }
        self.life -= delta;
    }
}
fn blast(pos: [f32; 2], vel: [f32; 2], spread: f32, count: usize, life: f32) -> impl Iterator<Item=Particle> {
    let mut rng = SmallRng::from_entropy();
    let xdst = Normal::new(vel[0] as f64, spread as f64);
    let ydst = Normal::new(vel[1] as f64, spread as f64);
    (0..count).map(move |_| {
        Particle {
            pos: pos,
            vel: [xdst.sample(&mut rng) as f32, ydst.sample(&mut rng) as f32],
            life: life,
            damp: 0.0,
            grav: [0.0, 400.0]
        }
    })
}
fn firework(pos: [f32; 2], vel: [f32; 2], spread: f32, count: usize, life: f32) -> impl Iterator<Item=Particle> {
    let mut rng = SmallRng::from_entropy();
    let xdst = Normal::new(vel[0] as f64, spread as f64);
    let ydst = Normal::new(vel[1] as f64, spread as f64);
    (0..count).map(move |_| {
        Particle {
            pos: pos,
            vel: [xdst.sample(&mut rng) as f32, ydst.sample(&mut rng) as f32],
            life: life,
            damp: 10.0,
            grav: [0.0, 0.0]
        }
    })
}
pub struct Overland {
    pub land_pos: Vector2<f32>,
    pub particles: Vec<Particle>,
    pub obstacles: Vec<([f32; 2], Box<Problem>)>,
    pub encounters: Vec<Box<Enemy>>,
}

impl Overland {
    fn new() -> Self {
        Overland {
            land_pos: [0.0, 0.0].into(),
            particles: vec![],
            obstacles: vec![
                ([650.0, 50.0], Box::new(TextProblem::new("536+329","865"))),
                ([250.0, -80.0], Box::new(TextProblem::new("3+3","6"))),
            ],
            encounters: vec![
                Box::new(Orb::new(1)),
                Box::new(Orb::new(3)),
                Box::new(Orb::new(5)),
            ],
        }
    }
}

pub struct Battle {
    pub land: Overland,
    pub enemy: Box<Enemy>,
    pub state: FighterState,
    pub enemy_props: EnemyProps,
    pub problem: Option<Box<Problem>>,
    pub particles: Vec<Particle>,
}

pub enum State {
    Title,
    Overland(Overland),
    Battle(Battle),
    Empty,
}

impl State {
    pub fn as_overland_mut(&mut self) -> Option<&mut Overland> {
        if let State::Overland(land) = self {
            Some(land)
        } else {
            None
        }
    }
    pub fn as_overland(&self) -> Option<&Overland> {
        if let State::Overland(land) = self {
            Some(land)
        } else {
            None
        }
    }
    pub fn as_battle(&self) -> Option<&Battle> {
        if let State::Battle(battle) = self {
            Some(battle)
        } else {
            None
        }
    }
}

pub struct Model {
    pub state: State,
    pub interval: IntervalService,
    pub console: ConsoleService,
    pub dir: Direction,
    pub text: TextBox,
    pub ctx: Context,
    pub _anim_task: Box<Task>,
}

pub enum Msg {
    AnimTick(f32),
    KeyDown(KeyDownEvent),
    KeyUp(KeyUpEvent),
}

fn eval_particles(particles: &mut Vec<Particle>, delta: f32) {
    for particle in particles.iter_mut() {
        particle.tick(delta);
    }
    for i in (0..particles.len()).rev() {
        if particles[i].life < 0.0 {
            particles.swap_remove(i);
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
        let callback = link.send_back(|e:KeyUpEvent| Msg::KeyUp(e));
        window().add_event_listener(move |e: KeyUpEvent| callback.emit(e));
        Model {
            state: State::Title,
            interval: interval,
            console: console,
            dir: Direction::new(),
            text: TextBox::new(),
            ctx: Context {
                anim_t: 0.0,
                time_damage: 0.0,
            },
            _anim_task: Box::new(handle),
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.text.movement = false;
        let mut rng = SmallRng::from_entropy();
        match msg {
            Msg::AnimTick(x) => {
                let new_state = match self.swap_state_out() {
                    State::Title => State::Title {
                    },
                    State::Overland(mut land) => {
                        eval_particles(&mut land.particles, x);
                        let speed = 100.0 * x;
                        let d = self.dir.direction();
                        land.land_pos[0] += d[0] * speed;
                        land.land_pos[1] += d[1] * speed;
                        if d != [0.0, 0.0] {
                            if 0.01 > rng.gen_range(0.0, 1.0) {
                                let items: Vec<usize> = (0..land.encounters.len()).collect();
                                if let Some(mut e) = items.choose(&mut rng).map(|&i| land.encounters.swap_remove(i)) {
                                    self.ctx.anim_t = 0.0;
                                    let enemy_props = e.get_properties();
                                    let b = Battle {
                                        land: land,
                                        enemy_props: enemy_props,
                                        problem: Some(e.generate_problem()),
                                        enemy: e,
                                        state: FighterState { health: 1.0 },
                                        particles: Vec::new()
                                    };
                                    State::Battle(b)
                                } else {
                                    State::Overland(land)
                                }
                            } else {
                                State::Overland(land)
                            }
                        } else {
                            State::Overland(land)
                        }
                    },
                    State::Battle(mut b) => {
                        eval_particles(&mut b.particles, x);
                        State::Battle(b)
                    },
                    x => x
                };
                self.swap_state_in(new_state);
                self.ctx.anim_t += x;
                self.ctx.time_damage += x;
                true
            },
            Msg::KeyDown(x) => {
                self.text.down(&x);
                self.dir.down(&x);
                self.console.log(&format!("{:?}", x.key()));
                let newstate = match self.swap_state_out() {
                    State::Title => {
                        if x.key() == "Enter" {
                            State::Overland(Overland::new())
                        } else {
                            State::Title
                        }
                    },
                    State::Overland(mut land) => {
                        self.text.movement = false;
                        if x.key() == "Enter" {
                            for i in (0..land.obstacles.len()).rev() {
                                if land.obstacles[i].1.test_correct(&self.text.text) {
                                    let (pos, _) = land.obstacles.swap_remove(i);
                                    land.particles.extend(firework([pos[0], -pos[1]], [0.0, 0.0], 500.0, 20, 0.75));
                                }
                            }
                            self.text = TextBox::new();
                        }
                        State::Overland(land)
                    },
                    State::Battle(mut battle) => {
                        if x.key() == "Enter" && battle.problem.is_some() {
                            let correct = if let Some(ref p) = battle.problem {
                                p.test_correct(&self.text.text)
                            } else {
                                false
                            };
                            if correct {
                                battle.enemy.damage(1.0);
                                self.ctx.time_damage = 0.0;
                                battle.particles.extend(blast([820.0, 250.0], [200.0, 200.0], 500.0, 10, 4.0));
                            } else {
                                battle.state.health -= 0.2;
                                battle.particles.extend(blast([180.0, 250.0], [-200.0, 200.0], 500.0, 10, 4.0));
                                self.console.log("INCORRECT");
                            }
                            self.text = TextBox::new();
                            if battle.enemy.get_state().health <= 0.0001 {
                                battle.problem = None;
                                State::Overland(battle.land)
                            } else {
                                battle.problem = Some(battle.enemy.generate_problem());
                                State::Battle(battle)
                            }
                        } else {
                            State::Battle(battle)
                        }
                    },
                    x => x,
                };
                self.swap_state_in(newstate);
                true
            },
            Msg::KeyUp(x) => {
                self.text.up(&x);
                self.dir.up(&x);
                self.console.log(&format!("{:?}", x));
                true
            },
        }
    }
}


impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <svg viewBox="0 0 1000 600", xmlns="http://www.w3.org/2000/svg",>
                {
                    match self.state {
                        State::Title => html! {
                            <circle cx=10, cy=0, r=100,></circle>
                        },
                        State::Battle(_) => self.battle(),
                        State::Overland(_) => self.overland(),
                        _ => html! {
                            <circle cx=500, cy=0, r=100,></circle>
                        },
                    }
                }
                </svg>
            </div>
        }
    }
}

impl Model {
    fn swap_state_out(&mut self) -> State {
        let mut s = State::Empty;
        std::mem::swap(&mut s, &mut self.state);
        s
    }
    fn swap_state_in(&mut self, s: State) {
        std::mem::replace(&mut self.state, s);
    }
    fn particles(&self, particles: &Vec<Particle>) -> Html<Self> {
        html! {
                    <g class="particles",>
                    {for particles.iter().map(|particle| {
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
        }
    }
    fn battle(&self) -> Html<Self> {
        let battle = self.state.as_battle().unwrap();
            let enemy_state = battle.enemy.get_state(); 
            html!{
                <g>
                    <image width=1000, height=800, x=0, y=-20, href="landscape_2.jpg",></image>
                    <g transform="translate(875, 400)",>{ battle.enemy_props.card.render(&self.ctx) }</g>
                    { self.particles(&battle.particles) }
                    <g transform="translate(750, 430)",>{ stats_card(&battle.enemy_props) }</g>
                    <rect class="problem_card", x=250, y=20, width=500, height=500, rx=10, ry=10,></rect>
                    <g transform="translate(500, 300) scale(5)",>
                        {
                            if let Some(ref p) = battle.problem {
                                p.render()
                            } else {
                                html! { <text>{"no problem"}</text> }
                            }
                        }
                    </g>
                    <g transform="translate(500, 560)",>
                        { self.text_box() }
                    </g>
                    <g transform="translate(0, 20)",>
                        { health_bar(battle.state.health, 475.0) }
                    </g>
                    <g transform="scale(-1.0, 1.0) translate(-1000, 20)",>
                        { health_bar(enemy_state.health, 475.0) }
                    </g>
                    <rect x={self.ctx.anim_t * 1000.0}, y=0, width=1000, height=600,></rect>
                </g>
            }

    }
    fn overland(&self) -> Html<Self> {
        let land = self.state.as_overland().unwrap();
        html! {
            <g transform="translate(500,300)",>
                <g transform={format!("translate({},{})", -land.land_pos[0], land.land_pos[1])},>
                    <image x=-1500, y=-1500, height=3000, width=3000, href="map.jpg",></image>
                    {for land.obstacles.iter().map(|x| {
                        html! {
                            <g transform={format!("translate({}, {})", x.0[0], -x.0[1])},>
                                <rect class="problem_card", x=-50, y=-50, width=100, height=100,></rect>
                                { x.1.render() }
                            </g>
                        }
                    })}
                    { self.particles(&land.particles) }
                </g>
                <circle class="person", r=10, x=0, y=0,></circle>
                <g transform="translate(0, 260)",>
                    { self.text_box() }
                </g>
            </g>
        }
    }
    fn text_box(&self) -> Html<Self> {
        html!{
            <g>
                <rect class="text_box", x=-250, y=-25, width=500, height=50, rx=10, ry=10,></rect>
                <text x=-240, y=12, class="answertext",>{&self.text.text}</text>
            </g>
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
