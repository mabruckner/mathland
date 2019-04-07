use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use rand::prelude::*;
use crate::Model;

pub trait Problem {
    fn render(&self) -> Html<Model>;
    fn test_correct(&self, answer: &str) -> bool;
    fn get_answer(&self) -> String;
}

pub struct TextProblem {
    pub problem: String,
    pub answer: String,
}

impl Problem for TextProblem {
    fn render(&self) -> Html<Model> {
        html! {
            <text class="textproblemtext",>{ &self.problem }</text>
        }
    }
    fn test_correct(&self, answer: &str) -> bool {
        self.answer == answer
    }
    fn get_answer(&self) -> String {
        self.answer.clone()
    }
}

impl TextProblem {
    pub fn new(problem: &str, answer: &str) -> Self{
        TextProblem {
            problem: problem.into(),
            answer: answer.into(),
        }
    }
}

pub fn gen_simple_add_sub(start: usize, end: usize) -> TextProblem {
    let mut rng = SmallRng::from_entropy();
    let (a, b) = (rng.gen_range(start, end), rng.gen_range(start, end));
    let c = a+b;
    if rng.gen::<bool>() {
        TextProblem {
            problem: format!("{}+{}=?", a, b),
            answer: format!("{}", c),
        }
    } else {
        TextProblem {
            problem: format!("{}-{}=?", c, a),
            answer: format!("{}", b),
        }
    }
}
