use crate::protocol::types::JsUInt;
use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScreenOrientation<'a> {
    #[serde(rename = "type")]
    pub orientation_type: &'a str,
    pub angle: JsUInt,
}

pub mod methods {
    use crate::protocol::page::Viewport;
    use crate::protocol::types::JsFloat;
    use crate::protocol::Method;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct SetDeviceMetricsOverride<'a> {
        pub width: JsFloat,
        pub height: JsFloat,
        pub device_scale_factor: JsFloat,
        pub mobile: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub scale: Option<JsFloat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub screen_width: Option<JsFloat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub screen_height: Option<JsFloat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub position_x: Option<JsFloat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub position_y: Option<JsFloat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dont_set_visible_size: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub screen_orientation: Option<super::ScreenOrientation<'a>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub viewport: Option<Viewport>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SetDeviceMetricsOverrideReturnObject {}
    impl Method for SetDeviceMetricsOverride<'_> {
        const NAME: &'static str = "Emulation.setDeviceMetricsOverride";
        type ReturnObject = SetDeviceMetricsOverrideReturnObject;
    }
}
