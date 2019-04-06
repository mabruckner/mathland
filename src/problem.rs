use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
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
