use failure::{Fail, Fallible};
use std::collections::HashSet;

use crate::browser::tab::keys::USKEYBOARD_LAYOUT;
use crate::protocol::input;
use crate::protocol::types::JsUInt;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

#[derive(Clone)]
pub struct Keyboard<'a> {
    pressed_keys: Arc<Mutex<HashSet<String>>>,
    modifiers: Arc<Mutex<u32>>,
    parent: &'a super::Tab,
}

impl<'a> Keyboard<'a> {
    pub fn new(parent: &'a super::Tab) -> Keyboard {
        Keyboard {
            pressed_keys: Arc::new(Mutex::new(HashSet::new())),
            modifiers: Arc::new(Mutex::new(0)),
            parent,
        }
    }
    pub fn down(&self, key: &str) -> Fallible<()> {
        let mut modifiers = self.modifiers.lock().unwrap();
        let description = get_key_definition(key, *modifiers)?;

        // See https://github.com/GoogleChrome/puppeteer/blob/62da2366c65b335751896afbb0206f23c61436f1/lib/Input.js#L52
        let key_down_event_type = if description.text.is_some() {
            "keyDown"
        } else {
            "rawKeyDown"
        };

        let mut pressed_keys = self.pressed_keys.lock().unwrap();
        let auto_repeat = pressed_keys.contains(description.code);
        pressed_keys.insert(description.code.clone().to_string());

        *modifiers |= self.modifier_bit(description.key);

        self.parent.call_method(input::methods::DispatchKeyEvent {
            event_type: key_down_event_type,
            modifiers: *modifiers,
            key: Some(description.key),
            text: description.text,
            code: Some(description.code),
            unmodified_text: description.text,
            location: description.location,
            is_keypad: description.location == 3,
            auto_repeat,
            windows_virtual_key_code: description.key_code,
            native_virtual_key_code: description.key_code,
        })?;

        Ok(())
    }

    pub fn up(&mut self, key: &str) -> Fallible<()> {
        let mut modifiers = self.modifiers.lock().unwrap();
        let description = get_key_definition(key, *modifiers)?;

        *modifiers &= !self.modifier_bit(description.key);

        let mut pressed_keys = self.pressed_keys.lock().unwrap();
        pressed_keys.remove(description.code);

        self.parent.call_method(input::methods::DispatchKeyEvent {
            event_type: "keyUp",
            modifiers: *modifiers,
            key: Some(description.key),
            text: None,
            code: Some(description.code),
            unmodified_text: None,
            location: description.location,
            is_keypad: false,
            auto_repeat: false,
            windows_virtual_key_code: description.key_code,
            native_virtual_key_code: description.key_code,
        })?;

        Ok(())
    }
    pub fn press(&mut self, key: &str, delay: Option<u64>) -> Fallible<()> {
        self.down(key)?;
        if let Some(delay) = delay {
            sleep(Duration::from_millis(delay));
        }
        self.up(key)?;
        Ok(())
    }
    pub fn send_character(&self, text: &str) -> Fallible<()> {
        self.parent
            .call_method(input::methods::InsertText { text })?;
        Ok(())
    }
    pub fn type_str(&mut self, string_to_type: &str) -> Fallible<()> {
        for c in string_to_type.split("") {
            if c == "" {
                continue;
            }
            if get_key_definition(c, *self.modifiers.lock().unwrap()).is_ok() {
                self.press(c, Some(25))?;
            } else {
                self.send_character(c)?;
            }
        }
        Ok(())
    }

    fn modifier_bit(&self, key: &str) -> JsUInt {
        match key {
            "Alt" => 1,
            "Control" => 2,
            "Meta" => 4,
            "Shift" => 8,
            _ => 0,
        }
    }
}

#[derive(Debug, Fail)]
#[fail(display = "Key not found: {}", key)]
pub struct KeyNotFoundError {
    key: String,
}

struct KeyDescription<'a> {
    pub key: &'a str,
    pub key_code: JsUInt,
    pub code: &'a str,
    pub text: Option<&'a str>,
    pub location: JsUInt,
}

fn get_key_definition(key: &str, modifiers: JsUInt) -> Fallible<KeyDescription> {
    if let Some(definition) = USKEYBOARD_LAYOUT
        .iter()
        .find(|key_definition| key_definition.key == key)
    {
        let shift = modifiers & 8;

        let key = if shift == 8 && definition.shift_key.is_some() {
            definition.shift_key.unwrap()
        } else {
            definition.key
        };

        let key_code = if shift == 8 && definition.shift_key_code.is_some() {
            definition.shift_key_code.unwrap()
        } else {
            definition.key_code
        };

        let text = if key.len() == 1 {
            Some(key)
        } else if definition.text.is_some() {
            definition.text
        } else if shift == 8 && definition.shift_text.is_some() {
            definition.shift_text
        } else {
            None
        };

        Ok(KeyDescription {
            key,
            key_code,
            code: definition.code,
            text,
            location: definition.location.unwrap_or(0),
        })
    } else {
        Err(KeyNotFoundError {
            key: key.to_string(),
        }
        .into())
    }
}
