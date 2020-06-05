use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod methods {
    use serde::{Deserialize, Serialize};

    use crate::protocol::input::MouseButton;
    use crate::protocol::types::{JsFloat, JsUInt};
    use crate::protocol::Method;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct DispatchMouseEvent<'a> {
        #[serde(rename = "type")]
        pub event_type: &'a str,
        pub x: JsFloat,
        pub y: JsFloat,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub modifiers: Option<JsUInt>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub button: Option<MouseButton>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub click_count: Option<JsUInt>,
    }
    impl<'a> Default for DispatchMouseEvent<'a> {
        fn default() -> Self {
            DispatchMouseEvent {
                event_type: "mouseMoved",
                x: 0.0,
                y: 0.0,
                modifiers: None,
                button: None,
                click_count: None,
            }
        }
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DispatchMouseEventReturnObject {}
    impl<'a> Method for DispatchMouseEvent<'a> {
        const NAME: &'static str = "Input.dispatchMouseEvent";
        type ReturnObject = DispatchMouseEventReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct DispatchKeyEvent<'a> {
        #[serde(rename = "type")]
        pub event_type: &'a str,
        pub modifiers: JsUInt,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub key: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub code: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "unmodifiedText")]
        pub unmodified_text: Option<&'a str>,
        pub location: JsUInt,
        #[serde(rename = "isKeypad")]
        pub is_keypad: bool,
        #[serde(rename = "autoRepeat")]
        pub auto_repeat: bool,
        pub windows_virtual_key_code: JsUInt,
        pub native_virtual_key_code: JsUInt,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DispatchKeyEventReturnObject {}
    impl<'a> Method for DispatchKeyEvent<'a> {
        const NAME: &'static str = "Input.dispatchKeyEvent";
        type ReturnObject = DispatchKeyEventReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct InsertText<'a> {
        pub text: &'a str,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct InsertTextReturnObject {}
    impl<'a> Method for InsertText<'a> {
        const NAME: &'static str = "Input.insertText";
        type ReturnObject = InsertTextReturnObject;
    }
}

#[derive(Clone, Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl Serialize for MouseButton {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match *self {
            MouseButton::Left => "left",
            MouseButton::Middle => "middle",
            MouseButton::Right => "right",
        })
    }
}

impl<'de> Deserialize<'de> for MouseButton {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "left" => MouseButton::Left,
            "middle" => MouseButton::Middle,
            "right" => MouseButton::Right,
            _ => MouseButton::Left,
        })
    }
}
