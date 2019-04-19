use yew::events::{KeyDownEvent, KeyUpEvent, IKeyboardEvent};


pub struct TextBox {
    pub text: String,
    pub cursor: usize,
    pub movement: bool,
}

impl TextBox {
    pub fn new() -> Self{
        TextBox {
            text: "".into(),
            cursor: 0,
            movement: true
        }
    }
    fn backspace(&mut self) {
        if self.cursor != 0 {
            self.text = String::from(&self.text[0..(self.cursor-1)]) + &self.text[self.cursor..self.text.len()];
            self.cursor -= 1;
        }
    }
    fn left(&mut self) {
        if self.movement {
            if self.cursor > 0 {
                self.cursor -= 1;
            }
        }
    }
    fn right(&mut self) {
        if self.movement {
            if self.text.chars().count() <= self.cursor {
                self.cursor = self.text.chars().count();
            } else {
                self.cursor += 1;
            }
        }
    }
    pub fn down(&mut self, e: &KeyDownEvent) {
        match e.key().as_str() {
            "ArrowRight" => self.right(),
            "ArrowLeft" => self.left(),
            "Backspace" => self.backspace(),
            x if x.len() == 1 => {
                self.text = String::from(&self.text[0..self.cursor]) + &x + &self.text[self.cursor..self.text.len()];
                self.cursor += 1;
            },
            _ => ()

        }
    }
    pub fn up(&mut self, _e: &KeyUpEvent) {
    }
}
