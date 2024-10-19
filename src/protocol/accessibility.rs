use crate::protocol::types::{JsFloat, JsUInt};
use serde::{Deserialize, Serialize};

pub type AXNodeId = String;

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AXNode {
    pub node_id: AXNodeId,
    pub ignored: bool,
    pub role: Option<AXValue>,
    pub name: Option<AXValue>,
    pub description: Option<AXValue>,
    pub value: Option<AXValue>,
    pub properties: Option<Vec<AXProperty>>,
    pub parent_id: Option<AXNodeId>,
    pub child_ids: Option<Vec<AXNodeId>>,
    pub backend_dom_node_id: Option<JsUInt>,
    pub frame_id: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AXValue {
    pub r#type: AXValueType,
    pub value: Option<serde_json::Value>,
    pub related_nodes: Option<Vec<AXRelatedNode>>,
    pub sources: Option<Vec<AXValueSource>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AXValueType {
    Boolean,
    Tristate,
    BooleanOrUndefined,
    Idref,
    IdrefList,
    Integer,
    Node,
    NodeList,
    Number,
    String,
    ComputedString,
    Token,
    TokenList,
    DomRelation,
    Role,
    InternalRole,
    ValueUndefined,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AXRelatedNode {
    pub backend_dom_node_id: JsUInt,
    pub id_ref: Option<String>,
    pub text: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AXValueSource {
    pub r#type: AXValueSourceType,
    pub value: Option<AXValue>,
    pub attribute: Option<String>,
    pub attribute_value: Option<serde_json::Value>,
    pub superseded: Option<bool>,
    pub native_source: Option<AXValueNativeSourceType>,
    pub native_source_value: Option<serde_json::Value>,
    pub invalid: Option<bool>,
    pub invalid_reason: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AXValueSourceType {
    Attribute,
    Implicit,
    Style,
    Contents,
    Placeholder,
    RelatedElement,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AXValueNativeSourceType {
    Figcaption,
    Label,
    Labelfor,
    Labelwrapped,
    Legend,
    Tablecaption,
    Title,
    Other,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AXProperty {
    pub name: AXPropertyName,
    pub value: AXValue,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AXPropertyName {
    Busy,
    Disabled,
    Editable,
    Focusable,
    Focused,
    Hidden,
    HiddenRoot,
    Invalid,
    Keyshortcuts,
    Settable,
    Roledescription,
    Live,
    Atomic,
    Relevant,
    Root,
    Autocomplete,
    HasPopup,
    Level,
    Multiselectable,
    Orientation,
    Multiline,
    Readonly,
    Required,
    Valuemin,
    Valuemax,
    Valuetext,
    Checked,
    Expanded,
    Modal,
    Pressed,
    Selected,
    Activedescendant,
    Controls,
    Describedby,
    Details,
    Errormessage,
    Flowto,
    Labelledby,
    Owns,
}

pub mod events {
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct LoadCompleteEvent {
        pub params: LoadCompleteParams,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct LoadCompleteParams {
        pub root: super::AXNode,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    pub struct NodesUpdatedEvent {
        pub params: NodesUpdatedParams,
    }

    #[derive(Deserialize, Debug, Clone, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct NodesUpdatedParams {
        pub nodes: Vec<super::AXNode>,
    }
}

pub mod methods {
    use serde::{Deserialize, Serialize};
    use crate::protocol::Method;
    use crate::protocol::types::JsUInt;
    use crate::protocol::dom::{NodeId};
    use super::AXNodeId;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetPartialAXTree {
        pub node_id: Option<NodeId>,
        pub backend_node_id: Option<NodeId>,
        pub object_id: Option<String>,
        pub fetch_relatives: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetPartialAXTreeReturnObject {
        pub nodes: Vec<super::AXNode>,
    }

    impl Method for GetPartialAXTree {
        const NAME: &'static str = "Accessibility.getPartialAXTree";
        type ReturnObject = GetPartialAXTreeReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetFullAXTree {
        pub max_depth: Option<JsUInt>,
        pub frame_id: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetFullAXTreeReturnObject {
        pub nodes: Vec<super::AXNode>,
    }

    impl Method for GetFullAXTree {
        const NAME: &'static str = "Accessibility.getFullAXTree";
        type ReturnObject = GetFullAXTreeReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Enable {}

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EnableReturnObject {}

    impl Method for Enable {
        const NAME: &'static str = "Accessibility.enable";
        type ReturnObject = EnableReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Disable {}

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DisableReturnObject {}

    impl Method for Disable {
        const NAME: &'static str = "Accessibility.disable";
        type ReturnObject = DisableReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryAXTree {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_id: Option<NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub backend_node_id: Option<NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub accessible_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub role: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QueryAXTreeReturnObject {
        pub nodes: Vec<super::AXNode>,
    }

    impl Method for QueryAXTree {
        const NAME: &'static str = "Accessibility.queryAXTree";
        type ReturnObject = QueryAXTreeReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetRootAXNode {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub frame_id: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetRootAXNodeReturnObject {
        pub node: super::AXNode,
    }

    impl Method for GetRootAXNode {
        const NAME: &'static str = "Accessibility.getRootAXNode";
        type ReturnObject = GetRootAXNodeReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetChildAXNodes {
        pub id: AXNodeId,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub frame_id: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetChildAXNodesReturnObject {
        pub nodes: Vec<super::AXNode>,
    }

    impl Method for GetChildAXNodes {
        const NAME: &'static str = "Accessibility.getChildAXNodes";
        type ReturnObject = GetChildAXNodesReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetAXNodeAndAncestors {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_id: Option<NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub backend_node_id: Option<NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetAXNodeAndAncestorsReturnObject {
        pub nodes: Vec<super::AXNode>,
    }

    impl Method for GetAXNodeAndAncestors {
        const NAME: &'static str = "Accessibility.getAXNodeAndAncestors";
        type ReturnObject = GetAXNodeAndAncestorsReturnObject;
    }
}