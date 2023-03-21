use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use willhook::{InputEvent, KeyboardEvent, KeyboardKey, KeyPress, MouseButton, MouseButtonPress, MouseEvent, MouseEventType, MousePressEvent, willhook};
use crate::input::Key::KeyQ;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Key {
    Left,
    KeyQ,
    KeyWindows,
}
#[derive(PartialEq, Hash, Debug, Copy, Clone, Default)]
pub struct InputState {
    pub active: bool,
    pub just_changed: bool,
}
impl InputState {
    pub fn new(active: bool, just_changed: bool) -> Self {
        Self {
            active,
            just_changed,
        }
    }
}
pub struct KeyboardMouseState {
    keys: Arc<Mutex<HashMap<Key, InputState>>>,
    thread: JoinHandle<()>,
}
impl KeyboardMouseState {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        keys.insert(Key::Left, InputState::default());
        keys.insert(Key::KeyQ, InputState::default());

        let keys = Arc::new(Mutex::new(keys));
        let keys_2 = keys.clone();

        let thread = thread::spawn(move || {
            let is_running = Arc::new(AtomicBool::new(true));
            let hook = willhook().unwrap();

            while is_running.load(Ordering::SeqCst) {
                if let Ok(ie) = hook.try_recv() {
                    let mut keys = keys.lock().unwrap();
                    match ie {
                        InputEvent::Keyboard(keyboard) => {
                            match keyboard {
                                KeyboardEvent { pressed, key, is_injected } => {
                                    match key {
                                        None => {}
                                        Some(key) => {
                                            match key {
                                                KeyboardKey::Q => {
                                                    match pressed {
                                                        KeyPress::Down(_) => {
                                                            keys.insert(Key::KeyQ, InputState::new(true, true));
                                                        }
                                                        KeyPress::Up(_) => {
                                                            keys.insert(Key::KeyQ, InputState::new(false, true));
                                                        }
                                                        KeyPress::Other(_) => {}
                                                    }
                                                }
                                                KeyboardKey::LeftWindows => {
                                                    match pressed {
                                                        KeyPress::Down(_) => {
                                                            keys.insert(Key::KeyWindows, InputState::new(true, true));
                                                        }
                                                        KeyPress::Up(_) => {
                                                            keys.insert(Key::KeyWindows, InputState::new(false, true));
                                                        }
                                                        KeyPress::Other(_) => {}
                                                    }
                                                }
                                                KeyboardKey::RightWindows => {
                                                    match pressed {
                                                        KeyPress::Down(_) => {
                                                            keys.insert(Key::KeyWindows, InputState::new(true, true));
                                                        }
                                                        KeyPress::Up(_) => {
                                                            keys.insert(Key::KeyWindows, InputState::new(false, true));
                                                        }
                                                        KeyPress::Other(_) => {}
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        InputEvent::Mouse(mouse) => {
                            match mouse {
                                MouseEvent { event, is_injected } => {
                                    match event {
                                        MouseEventType::Press(press) => {
                                            match press {
                                                MousePressEvent { pressed, button } => {
                                                    match button {
                                                        MouseButton::Left(click) => {
                                                            match pressed {
                                                                MouseButtonPress::Down => {
                                                                    keys.insert(Key::Left, InputState::new(true, true));
                                                                }
                                                                MouseButtonPress::Up => {
                                                                    keys.insert(Key::Left, InputState::new(false, true));
                                                                }
                                                                MouseButtonPress::Other(_) => {}
                                                            }
                                                        }
                                                        MouseButton::Right(_) => {}
                                                        MouseButton::Middle(_) => {}
                                                        MouseButton::X1(_) => {}
                                                        MouseButton::X2(_) => {}
                                                        MouseButton::UnkownX(_) => {}
                                                        MouseButton::Other(_) => {}
                                                    }
                                                }
                                            }
                                        }
                                        MouseEventType::Move(_) => {}
                                        MouseEventType::Wheel(_) => {}
                                        MouseEventType::Other(_) => {}
                                    }
                                }
                            }
                        }
                        InputEvent::Other(_) => {}
                    }
                }
            }

        });


        Self {
            keys: keys_2,
            thread,
        }
    }
    pub fn get_input(&mut self, key: Key) -> InputState {
        let mut keys = self.keys.lock().unwrap();
        let return_state = *keys.get(&key).expect("key should be in keys map");
        if return_state.just_changed {
            keys.get_mut(&key).unwrap().just_changed = false;
        }
        return_state
    }
}