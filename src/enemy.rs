use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use crate::problem::*;
use crate::Model;
use crate::context::*;
use std::fmt::Debug;

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

impl EnemyCard for CircleCard {
    fn render(&self, ctx: &Context) -> Html<Model> {
        let float = ctx.anim_t.sin() * 0.1 + 1.0;
        let unfloat = ctx.anim_t.sin() * (-0.1) + 1.0;
        html!{
            <g class="circle_card",>
            { shadow(100.0*unfloat, 0.0, 0.0) }
                <circle cx=0, cy ={-100.0*float}, r=80,></circle>
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
    health: f64,
}

enum EnemyAction {
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
}

impl Orb {
    pub fn new() -> Self {
        Orb {
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
        EnemyProps {
            level: "1".into(),
            class: "spheroid".into(),
            name: "Circle".into(),
            card: Box::new(CircleCard()),
        }
    }
    fn damage(&mut self, amount: f64) {
        self.state.health -= 0.2*amount;
    }
    fn act(&mut self, delta: f64) -> Option<EnemyAction> {
        None
    }
    fn generate_problem(&mut self) -> Box<Problem> {
        Box::new(TextProblem {
            problem: "2 + 2".into(),
            answer: "4".into()
        })
    }
}
