use crate::browser::transport::{SessionId, Transport};
use crate::protocol;
use crate::protocol::input;
use crate::protocol::input::MouseButton;
use crate::protocol::types::{JsFloat, JsUInt};
use failure::{Fail, Fallible};
use log::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Mouse {
    keyboard_modifiers: Arc<Mutex<u32>>,
    transport: Arc<Transport>,
    session_id: SessionId,
    x: Arc<Mutex<JsFloat>>,
    y: Arc<Mutex<JsFloat>>,
    button: Arc<Mutex<Option<MouseButton>>>,
}

impl Mouse {
    pub fn new(
        keyboard_modifiers: Arc<Mutex<u32>>,
        transport: Arc<Transport>,
        session_id: SessionId,
    ) -> Mouse {
        Mouse {
            keyboard_modifiers,
            transport,
            session_id,
            x: Arc::new(Mutex::new(0.0)),
            y: Arc::new(Mutex::new(0.0)),
            button: Arc::new(Mutex::new(None)),
        }
    }
    pub fn mouse_move(&self, x: JsFloat, y: JsFloat, steps: usize) -> Fallible<()> {
        let mut from_x = self.x.lock().unwrap();
        let mut from_y = self.y.lock().unwrap();
        let mouse_button = self.button.lock().unwrap();
        for step in 1..steps+1 {
            let method = input::methods::DispatchMouseEvent {
                event_type: "mouseMoved",
                x: *from_x + (x - *from_x) * (step as JsFloat / steps as JsFloat ) as JsFloat,
                y: *from_y + (y - *from_y) * (step as JsFloat / steps as JsFloat ) as JsFloat,
                modifiers: Some(*self.keyboard_modifiers.lock().unwrap()),
                button: mouse_button.clone(),
                click_count: None,
            };
            self.call_method(method)?;
        }
        *from_x = x;
        *from_y = y;
        Ok(())
    }
    pub fn click(
        &self,
        x: JsFloat,
        y: JsFloat,
        button: MouseButton,
        click_count: usize,
        delay: usize,
    ) -> Fallible<()> {
        self.mouse_move(x, y, 1)?;
        self.down(button.clone(), click_count)?;
        thread::sleep(Duration::from_millis(delay as u64));
        self.up(button, click_count)?;
        Ok(())
    }
    pub fn down(&self, button: MouseButton, click_count: usize) -> Fallible<()> {
        let mut mouse_button = self.button.lock().unwrap();
        let x = self.x.lock().unwrap();
        let y = self.y.lock().unwrap();
        *mouse_button = Some(button.clone());
        self.call_method(input::methods::DispatchMouseEvent {
            event_type: "mousePressed",
            x: *x,
            y: *y,
            modifiers: Some(*self.keyboard_modifiers.lock().unwrap()),
            button: Some(button),
            click_count: Some(click_count as JsUInt),
        })?;
        Ok(())
    }
    pub fn up(&self, button: MouseButton, click_count: usize) -> Fallible<()> {
        let mut mouse_button = self.button.lock().unwrap();
        let x = self.x.lock().unwrap();
        let y = self.y.lock().unwrap();
        *mouse_button = None;
        self.call_method(input::methods::DispatchMouseEvent {
            event_type: "mouseReleased",
            x: *x,
            y: *y,
            modifiers: Some(*self.keyboard_modifiers.lock().unwrap()),
            button: Some(button.into()),
            click_count: Some(click_count as JsUInt),
        })?;
        Ok(())
    }
    fn call_method<C>(&self, method: C) -> Fallible<C::ReturnObject>
    where
        C: protocol::Method + serde::Serialize + std::fmt::Debug,
    {
        trace!("Calling method: {:?}", method);
        let result = self
            .transport
            .call_method_on_target(self.session_id.clone(), method);
        let mut result_string = format!("{:?}", result);
        result_string.truncate(70);
        trace!("Got result: {:?}", result_string);
        result
    }
}
