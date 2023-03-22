use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use willhook::{InputEvent, KeyboardEvent, KeyboardKey, KeyPress, MouseButton, MouseButtonPress, MouseEvent, MouseEventType, MousePressEvent, willhook};
use crate::input::Key::Q;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Key {
    MouseLeft,
    Windows,
    Backspace,
    ArrowDown,
    ArrowUp,
    Enter,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z
}
impl Key {
    pub fn as_str<'a>(&self) -> &'a str {
        match self {
            Key::MouseLeft => "MouseLeft",
            Key::Windows => "Windows",
            Key::Backspace => "Backspace",
            Key::A => "a",
            Key::B => "b",
            Key::C => "c",
            Key::D => "d",
            Key::E => "e",
            Key::F => "f",
            Key::G => "g",
            Key::H => "h",
            Key::I => "i",
            Key::J => "j",
            Key::K => "k",
            Key::L => "l",
            Key::M => "m",
            Key::N => "n",
            Key::O => "o",
            Key::P => "p",
            Key::Q => "q",
            Key::R => "r",
            Key::S => "s",
            Key::T => "t",
            Key::U => "u",
            Key::V => "v",
            Key::W => "w",
            Key::X => "x",
            Key::Y => "y",
            Key::Z => "z",
            _ => panic!()
        }
    }
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
fn keyboard_press(key: Key, key_press: KeyPress, keys: &mut MutexGuard<HashMap<Key, InputState>>) {
    match key_press {
        KeyPress::Down(_) => {
            keys.insert(key, InputState::new(true, true));
        }
        KeyPress::Up(_) => {
            keys.insert(key, InputState::new(false, true));
        }
        KeyPress::Other(_) => {}
    }
}
pub struct KeyboardMouseState {
    keys: Arc<Mutex<HashMap<Key, InputState>>>,
    thread: JoinHandle<()>,
}
impl KeyboardMouseState {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        keys.insert(Key::MouseLeft, InputState::default());
        keys.insert(Key::Windows, InputState::default());
        keys.insert(Key::Backspace, InputState::default());
        keys.insert(Key::Enter, InputState::default());
        keys.insert(Key::ArrowUp, InputState::default());
        keys.insert(Key::ArrowDown, InputState::default());
        keys.insert(Key::A, InputState::default());
        keys.insert(Key::B, InputState::default());
        keys.insert(Key::C, InputState::default());
        keys.insert(Key::D, InputState::default());
        keys.insert(Key::E, InputState::default());
        keys.insert(Key::F, InputState::default());
        keys.insert(Key::G, InputState::default());
        keys.insert(Key::H, InputState::default());
        keys.insert(Key::I, InputState::default());
        keys.insert(Key::J, InputState::default());
        keys.insert(Key::K, InputState::default());
        keys.insert(Key::L, InputState::default());
        keys.insert(Key::M, InputState::default());
        keys.insert(Key::N, InputState::default());
        keys.insert(Key::O, InputState::default());
        keys.insert(Key::P, InputState::default());
        keys.insert(Key::Q, InputState::default());
        keys.insert(Key::R, InputState::default());
        keys.insert(Key::S, InputState::default());
        keys.insert(Key::T, InputState::default());
        keys.insert(Key::U, InputState::default());
        keys.insert(Key::V, InputState::default());
        keys.insert(Key::W, InputState::default());
        keys.insert(Key::X, InputState::default());
        keys.insert(Key::Y, InputState::default());
        keys.insert(Key::Z, InputState::default());

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
                                                KeyboardKey::LeftWindows => {
                                                    keyboard_press(Key::Windows, pressed, &mut keys);
                                                }
                                                KeyboardKey::RightWindows => {
                                                    keyboard_press(Key::Windows, pressed, &mut keys);
                                                }
                                                KeyboardKey::Enter => keyboard_press(Key::Enter, pressed, &mut keys),
                                                KeyboardKey::ArrowUp => keyboard_press(Key::ArrowUp, pressed, &mut keys),
                                                KeyboardKey::ArrowDown => keyboard_press(Key::ArrowDown, pressed, &mut keys),
                                                KeyboardKey::BackSpace => keyboard_press(Key::Backspace, pressed, &mut keys),
                                                KeyboardKey::A => keyboard_press(Key::A, pressed, &mut keys),
                                                KeyboardKey::B => keyboard_press(Key::B, pressed, &mut keys),
                                                KeyboardKey::C => keyboard_press(Key::C, pressed, &mut keys),
                                                KeyboardKey::D => keyboard_press(Key::D, pressed, &mut keys),
                                                KeyboardKey::E => keyboard_press(Key::E, pressed, &mut keys),
                                                KeyboardKey::F => keyboard_press(Key::F, pressed, &mut keys),
                                                KeyboardKey::G => keyboard_press(Key::G, pressed, &mut keys),
                                                KeyboardKey::H => keyboard_press(Key::H, pressed, &mut keys),
                                                KeyboardKey::I => keyboard_press(Key::I, pressed, &mut keys),
                                                KeyboardKey::J => keyboard_press(Key::J, pressed, &mut keys),
                                                KeyboardKey::K => keyboard_press(Key::K, pressed, &mut keys),
                                                KeyboardKey::L => keyboard_press(Key::L, pressed, &mut keys),
                                                KeyboardKey::M => keyboard_press(Key::M, pressed, &mut keys),
                                                KeyboardKey::N => keyboard_press(Key::N, pressed, &mut keys),
                                                KeyboardKey::O => keyboard_press(Key::O, pressed, &mut keys),
                                                KeyboardKey::P => keyboard_press(Key::P, pressed, &mut keys),
                                                KeyboardKey::Q => keyboard_press(Key::Q, pressed, &mut keys),
                                                KeyboardKey::R => keyboard_press(Key::R, pressed, &mut keys),
                                                KeyboardKey::S => keyboard_press(Key::S, pressed, &mut keys),
                                                KeyboardKey::T => keyboard_press(Key::T, pressed, &mut keys),
                                                KeyboardKey::U => keyboard_press(Key::U, pressed, &mut keys),
                                                KeyboardKey::V => keyboard_press(Key::V, pressed, &mut keys),
                                                KeyboardKey::W => keyboard_press(Key::W, pressed, &mut keys),
                                                KeyboardKey::X => keyboard_press(Key::X, pressed, &mut keys),
                                                KeyboardKey::Y => keyboard_press(Key::Y, pressed, &mut keys),
                                                KeyboardKey::Z => keyboard_press(Key::Z, pressed, &mut keys),
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
                                                                    keys.insert(Key::MouseLeft, InputState::new(true, true));
                                                                }
                                                                MouseButtonPress::Up => {
                                                                    keys.insert(Key::MouseLeft, InputState::new(false, true));
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