use yew::{html, Html};
use crate::problem::*;
use crate::Model;
use crate::context::*;
use rand::prelude::*;

pub trait EnemyCard {
    fn render(&self, ctx: &Context) -> Html<Model>;
}

#[derive(Debug, Copy, Clone)]
pub struct CircleCard();

fn shadow(size: f32, x: f32, y: f32) -> Html<Model> {
    html!{
        <ellipse class="shadow", cx=x, cy=y, rx={size/2.0}, ry={size/8.0},></ellipse>
    }
}

fn damage_box(ctx: &Context) -> Html<Model> {
    if ctx.time_damage < 0.25 {
        let mut rng = SmallRng::from_entropy();
        let x = rng.gen_range(-40.0, 40.0);
        let y = rng.gen_range(-150.0, -50.0);
        let w = rng.gen_range(10.0, 200.0);
        let h = rng.gen_range(5.0, 30.0);
        html!{
            <rect class="black", x={x-w/2.0}, y={y-h/2.0}, width=w, height=h,></rect>
        }
    } else {
        html!{
            <g></g>
        }
    }
}

impl EnemyCard for CircleCard {
    fn render(&self, ctx: &Context) -> Html<Model> {
        let float = ctx.anim_t.sin() * 0.1 + 1.0;
        let unfloat = ctx.anim_t.sin() * (-0.1) + 1.0;
        html!{
            <g class="circle_card",>
            { shadow(100.0*unfloat, 0.0, 0.0) }
                <circle cx=0, cy ={-100.0*float}, r=80,></circle>
                {damage_box(ctx)}
            </g>
        }
    }
}

pub struct OrbCard();

impl EnemyCard for OrbCard {
    fn render(&self, ctx: &Context) -> Html<Model> {
        let float = ctx.anim_t.sin() * 0.1 + 1.0;
        let unfloat = ctx.anim_t.sin() * (-0.1) + 1.0;
        let rot = ctx.anim_t*1.0;
        html!{
            <g class="circle_card",>
            { shadow(100.0*unfloat, 0.0, 0.0) }
            {for (0..4).map(|i| {
                html!{
                    <ellipse cx=0, cy={-100.0*float}, ry=80, rx={(rot+(i as f32)*3.141/4.0).sin().abs()*80.0},></ellipse>
                }
            })}
                {damage_box(ctx)}
            </g>
        }
    }
}

pub struct EnemyProps {
    pub level: String,
    pub class: String,
    pub name: String,
    pub card: Box<EnemyCard>,
}

#[derive(Debug, Clone)]
pub struct FighterState {
    pub health: f64,
}

pub enum EnemyAction {
    Attack(f64),
}

pub trait Enemy {
    fn get_state(&self) -> FighterState;
    fn get_properties(&self) -> EnemyProps;
    fn damage(&mut self, amount: f64) -> ();
    fn act(&mut self, delta: f64) -> Option<EnemyAction>;
    fn generate_problem(&mut self) -> Box<Problem>;
}


pub struct Orb {
    state: FighterState,
    level: usize,
}

impl Orb {
    pub fn new(level: usize) -> Self {
        Orb {
            level: level,
            state: FighterState {
                health: 1.0
            }
        }
    }
}

impl Enemy for Orb {
    fn get_state(&self) -> FighterState {
        self.state.clone()
    }
    fn get_properties(&self) -> EnemyProps {
        let (name, card): (_, Box<EnemyCard>) = match self.level {
            0...3 => ("Circle", Box::new(CircleCard())),
            _ => ("Orb", Box::new(OrbCard())),
        };
        EnemyProps {
            level: format!("{}", self.level),
            class: "spheroid".into(),
            name: name.into(),
            card: card,
        }
    }
    fn damage(&mut self, amount: f64) {
        self.state.health -= 0.2*amount;
    }
    fn act(&mut self, _delta: f64) -> Option<EnemyAction> {
        None
    }
    fn generate_problem(&mut self) -> Box<Problem> {
        Box::new(gen_simple_add_sub(self.level*2, self.level*8))
    }
}
