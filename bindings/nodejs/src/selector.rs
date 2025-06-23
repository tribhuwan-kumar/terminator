use napi::bindgen_prelude::FromNapiValue;
use napi_derive::napi;
use std::collections::BTreeMap;
use terminator::selector::Selector as TerminatorSelector;

/// Selector for locating UI elements. Provides a typed alternative to the string based selector API.
#[napi(js_name = "Selector")]
pub struct Selector {
    pub(crate) inner: TerminatorSelector,
}

impl From<TerminatorSelector> for Selector {
    fn from(sel: TerminatorSelector) -> Self {
        Selector { inner: sel }
    }
}

impl From<&Selector> for TerminatorSelector {
    fn from(sel: &Selector) -> Self {
        sel.inner.clone()
    }
}

impl FromNapiValue for Selector {
    unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        napi_val: napi::sys::napi_value,
    ) -> napi::Result<Self> {
        let mut result = std::ptr::null_mut();
        let status = napi::sys::napi_get_value_external(env, napi_val, &mut result);
        if status != napi::sys::Status::napi_ok {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                "Failed to get external value for Selector".to_string(),
            ));
        }
        Ok(std::ptr::read(result as *const Selector))
    }
}

#[napi]
impl Selector {
    /// Create a selector that matches elements by their accessibility `name`.
    #[napi(factory)]
    pub fn name(name: String) -> Self {
        Selector::from(TerminatorSelector::Name(name))
    }

    /// Create a selector that matches elements by role (and optionally name).
    #[napi(factory)]
    pub fn role(role: String, name: Option<String>) -> Self {
        Selector::from(TerminatorSelector::Role { role, name })
    }

    /// Create a selector that matches elements by accessibility `id`.
    #[napi(factory, js_name = "id")]
    pub fn id_factory(id: String) -> Self {
        Selector::from(TerminatorSelector::Id(id))
    }

    /// Create a selector that matches elements by the text they display.
    #[napi(factory)]
    pub fn text(text: String) -> Self {
        Selector::from(TerminatorSelector::Text(text))
    }

    /// Create a selector from an XPath-like path string.
    #[napi(factory)]
    pub fn path(path: String) -> Self {
        Selector::from(TerminatorSelector::Path(path))
    }

    /// Create a selector that matches elements by a native automation id (e.g., AutomationID on Windows).
    #[napi(factory, js_name = "nativeId")]
    pub fn native_id(id: String) -> Self {
        Selector::from(TerminatorSelector::NativeId(id))
    }

    /// Create a selector that matches elements by their class name.
    #[napi(factory, js_name = "className")]
    pub fn class_name(name: String) -> Self {
        Selector::from(TerminatorSelector::ClassName(name))
    }

    /// Create a selector from an arbitrary attribute map.
    #[napi(factory)]
    pub fn attributes(attributes: std::collections::HashMap<String, String>) -> Self {
        let map: BTreeMap<String, String> = attributes.into_iter().collect();
        Selector::from(TerminatorSelector::Attributes(map))
    }

    /// Chain another selector onto this selector.
    #[napi]
    pub fn chain(&self, other: &Selector) -> Selector {
        Selector::from(TerminatorSelector::Chain(vec![
            self.inner.clone(),
            other.inner.clone(),
        ]))
    }

    /// Filter by visibility.
    #[napi]
    pub fn visible(&self, is_visible: bool) -> Selector {
        Selector::from(TerminatorSelector::Chain(vec![
            self.inner.clone(),
            TerminatorSelector::Visible(is_visible),
        ]))
    }

    #[napi]
    pub fn to_string(&self) -> String {
        format!("{:?}", self.inner)
    }
}
