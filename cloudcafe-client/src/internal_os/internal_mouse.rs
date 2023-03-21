use std::sync::{Arc, Mutex};
use crate::input::KeyboardMouseState;
use crate::values::{IVec2, UVec2};
use crate::windows_bindings::{get_cursor_pos, set_cursor_pos};

pub struct IMouse {
    pub lock_cursor: bool,
    pos: IVec2,
    pub delta_pos: IVec2,
    cursor_lock_position: IVec2,
}
impl IMouse {
    pub fn new(cursor_lock_position: IVec2) -> Self {
        let cursor_pos = get_cursor_pos();
        Self {
            lock_cursor: false,
            pos: IVec2::from([cursor_pos.x, cursor_pos.y]),
            delta_pos: IVec2::from([0, 0]),
            cursor_lock_position,
        }
    }
    pub fn tick(&mut self) {
        let new_pos = self.pos();
        self.delta_pos.x = new_pos.x - self.pos.x;
        self.delta_pos.y = new_pos.y - self.pos.y;
        self.pos = new_pos;
        if self.lock_cursor {
            self.pos = self.cursor_lock_position;
            self.set_pos(self.cursor_lock_position);
        }
    }
    pub fn pos(&self) -> IVec2 {
        let pos = get_cursor_pos();
        IVec2::from([pos.x, pos.y])
    }
    pub fn set_pos(&mut self, pos: IVec2) {
        //self.pos = pos;
        set_cursor_pos(pos.x, pos.y);
    }
}