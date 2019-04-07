#![recursion_limit="256"]

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

pub struct Model {
    pub interval: IntervalService,
    pub console: ConsoleService,
    pub dir: Direction,
    pub text: TextBox,
    pub problem: Option<Box<Problem>>,
    pub enemy: Option<Box<Enemy>>,
    pub enemy_props: EnemyProps,
    pub ctx: Context,
    pub particles: Vec<Particle>,
    pub state: FighterState,
    pub land_pos: [f32; 2],
    pub obstacles: Vec<([f32; 2], Box<Problem>)>,
    pub encounters: Vec<Box<Enemy>>,
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
    fn down(&mut self, evt: &KeyDownEvent) {
        self.set(&evt.key(), true);
    }
    fn up(&mut self, evt: &KeyUpEvent) {
        self.set(&evt.key(), false);
    }
    fn set(&mut self, code: &str, val: bool) {
        match code {
            "ArrowUp" => { self.up = val; },
            "ArrowDown" => { self.down = val; },
            "ArrowLeft" => { self.left = val; },
            "ArrowRight" => { self.right = val; },
            _ => ()
        }
    }
    fn direction(&self) -> [f32; 2] {
        let x = match (self.right, self.left) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };
        let y = match (self.up, self.down) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };
        [x, y]
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
        let handle = interval.spawn(Duration::from_millis(50), link.send_back(|_| Msg::AnimTick(0.05)));
        let mut console = ConsoleService::new();
        console.log("Starting up");
        let callback = link.send_back(|e:KeyDownEvent| Msg::KeyDown(e));
        window().add_event_listener(move |e: KeyDownEvent| callback.emit(e));
        let callback = link.send_back(|e:KeyUpEvent| Msg::KeyUp(e));
        window().add_event_listener(move |e: KeyUpEvent| callback.emit(e));
        let e = Box::new(Orb::new(4));
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
            enemy: None,
            enemy_props: props,
            ctx: Context {
                anim_t: 0.0
            },
            particles: Vec::new(),
            state: FighterState { 
                health: 1.0,
            },
            land_pos: [0.0; 2],
            _anim_task: Box::new(handle),
            encounters: vec![
                Box::new(Orb::new(1)),
                Box::new(Orb::new(3)),
                Box::new(Orb::new(5)),
            ],
            obstacles: vec![
                ([650.0, 50.0], Box::new(TextProblem::new("536+329","865"))),
                ([250.0, -80.0], Box::new(TextProblem::new("3+3","6"))),
            ],
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        self.text.movement = self.enemy.is_some();
        let mut rng = SmallRng::from_entropy();
        match msg {
            Msg::AnimTick(x) => {
                if self.enemy.is_none() {
                    let speed = 100.0 * x;
                    let d = self.dir.direction();
                    if d != [0.0, 0.0] {
                        if 0.01 > rng.gen_range(0.0, 1.0) {
                            let items: Vec<usize> = (0..self.encounters.len()).collect();
                            if let Some(mut e) = rng.choose(&items).map(|&i| self.encounters.swap_remove(i)) {
                                self.enemy_props = e.get_properties();
                                self.problem = Some(e.generate_problem());
                                self.enemy = Some(e);
                                self.state.health = 1.0;
                                self.ctx.anim_t = 0.0;
                            }
                        }
                    }
                    self.land_pos[0] += d[0] * speed;
                    self.land_pos[1] += d[1] * speed;
                } else {
                    let health = if let Some(ref x) = self.enemy {
                        x.get_state().health
                    } else {
                        0.0
                    };
                    if health <= 0.0 {
                        self.enemy = None;
                    }
                }
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
                if x.key() == "Enter" && self.problem.is_some() && self.enemy.is_some() {
                    let correct = if let Some(ref p) = self.problem {
                        p.test_correct(&self.text.text)
                    } else {
                        false
                    };
                    if self.enemy.is_some() {
                    if correct {
                        if let Some(ref mut enemy) = self.enemy {
                            enemy.damage(1.0);
                        }

                        self.blast([820.0, 250.0], [200.0, 200.0], 500.0, 10, 4.0);
                    } else {
                        self.state.health -= 0.2;
                        self.blast([180.0, 250.0], [-200.0, 200.0], 500.0, 10, 4.0);
                        self.console.log("INCORRECT");
                        // incorrect
                    }
                    if let Some(ref mut enemy) = self.enemy {
                       self.problem = Some(enemy.generate_problem());
                    }
                    }
                    self.text = TextBox::new();
                } else if x.key() == "Enter" && self.enemy.is_none() {
                    for i in (0..self.obstacles.len()).rev() {
                        if self.obstacles[i].1.test_correct(&self.text.text) {
                            let (pos, _) = self.obstacles.swap_remove(i);
                            self.firework([pos[0], -pos[1]], [0.0, 0.0], 500.0, 20, 0.75);
                        }
                    }
                    self.text = TextBox::new();
                } else {
                    self.text.down(&x);
                    self.dir.down(&x);
                    self.console.log(&format!("{:?}", x.key()));
                }
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
                    if self.enemy.is_some() {
                        self.battle()
                    } else {
                        self.overland()
                    }
                }
                </svg>
            </div>
        }
    }
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
                damp: 0.0,
                grav: [0.0, 400.0]
            });
        }
    }
    fn firework(&mut self, pos: [f32; 2], vel: [f32; 2], spread: f32, count: usize, life: f32) {
        let mut rng = SmallRng::from_entropy();
        let xdst = Normal::new(vel[0] as f64, spread as f64);
        let ydst = Normal::new(vel[1] as f64, spread as f64);
        for i in 0..count {
            self.particles.push(Particle {
                p_type: PType::Spark,
                pos: pos,
                vel: [xdst.sample(&mut rng) as f32, ydst.sample(&mut rng) as f32],
                life: life,
                damp: 10.0,
                grav: [0.0, 0.0]
            });
        }
    }
    fn particles(&self) -> Html<Self> {
        html! {
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
        }
    }
    fn battle(&self) -> Html<Self> {
        if let Some(ref enemy) = self.enemy {
            let enemy_state = enemy.get_state(); 
            html!{
                <g>
                    <image width=1000, height=800, x=0, y=-20, href="landscape_2.jpg",></image>
                    <g transform="translate(875, 400)",>{ self.enemy_props.card.render(&self.ctx) }</g>
                    { self.particles() }
                    <g transform="translate(750, 430)",>{ stats_card(&self.enemy_props) }</g>
                    <rect class="problem_card", x=250, y=20, width=500, height=500, rx=10, ry=10,></rect>
                    <g transform="translate(500, 300) scale(5)",>
                        {
                            if let Some(ref p) = self.problem {
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
                        { health_bar(self.state.health, 475.0) }
                    </g>
                    <g transform="scale(-1.0, 1.0) translate(-1000, 20)",>
                        { health_bar(enemy_state.health, 475.0) }
                    </g>
                    <rect x={self.ctx.anim_t * 1000.0}, y=0, width=1000, height=600,></rect>
                </g>
            }

        } else {
            html! {
                <text>{"what"}</text>
            }
        }
    }
    fn overland(&self) -> Html<Self> {
        html! {
            <g transform="translate(500,300)",>
                <g transform={format!("translate({},{})", -self.land_pos[0], self.land_pos[1])},>
                    <image x=-1500, y=-1500, height=3000, width=3000, href="map.jpg",></image>
                {for self.obstacles.iter().map(|x| {
                                                       html! {
                                                           <g transform={format!("translate({}, {})", x.0[0], -x.0[1])},>
                                                               <rect class="problem_card", x=-50, y=-50, width=100, height=100,></rect>
                                                           { x.1.render() }
                                                            </g>
                                                       }
                                                   })}
                    { self.particles() }
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
