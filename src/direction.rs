use yew::events::{KeyDownEvent, KeyUpEvent, IKeyboardEvent};

pub struct Direction {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Direction {
    pub fn new() -> Self {
        Direction {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
    pub fn down(&mut self, evt: &KeyDownEvent) {
        self.set(&evt.key(), true);
    }
    pub fn up(&mut self, evt: &KeyUpEvent) {
        self.set(&evt.key(), false);
    }
    pub fn set(&mut self, code: &str, val: bool) {
        match code {
            "ArrowUp" => { self.up = val; },
            "ArrowDown" => { self.down = val; },
            "ArrowLeft" => { self.left = val; },
            "ArrowRight" => { self.right = val; },
            _ => ()
        }
    }
    pub fn direction(&self) -> [f32; 2] {
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

