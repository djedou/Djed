use super::{Attributes, Listener, Listeners, Patch, Transformer, VDiff, VList, VNode};
use crate::djed::{AnyScope, NodeRef};
use crate::utils::document;
use log::warn;
use std::borrow::Cow;
use std::cmp::PartialEq;
use std::rc::Rc;
use gloo::events::EventListener;
use std::ops::Deref;
use wasm_bindgen::JsCast;
use web_sys::{
    Element, HtmlInputElement as InputElement, HtmlTextAreaElement as TextAreaElement, HtmlButtonElement
};


/// SVG namespace string used for creating svg elements
pub const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// Default namespace for html elements
//pub const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

/// Used to improve performance of runtime element checks
#[derive(Clone, Copy, Debug, PartialEq)]
enum ElementType {
    Input,
    Textarea,
    Button,
    Other,
}

impl ElementType {
    fn from_tag(tag: &str) -> Self {
        match tag.to_ascii_lowercase().as_str() {
            "input" => Self::Input,
            "textarea" => Self::Textarea,
            "button" => Self::Button,
            _ => Self::Other,
        }
    }
}

/// A type for a virtual
/// [Element](https://developer.mozilla.org/en-US/docs/Web/API/Element)
/// representation.
#[derive(Debug)]
pub struct VTag {
    /// A tag of the element.
    tag: Cow<'static, str>,
    /// Type of element.
    element_type: ElementType,
    /// A reference to the `Element`.
    pub reference: Option<Element>,
    /// List of attached listeners.
    pub listeners: Listeners,
    /// List of attributes.
    pub attributes: Attributes,
    /// List of children nodes
    pub children: VList,
    /// Contains a value of an
    /// [InputElement](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input).
    pub value: Option<String>,
    /// Contains
    /// [kind](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#Form_%3Cinput%3E_types)
    /// value of an `InputElement`.
    pub kind: Option<String>,
    /// Represents `checked` attribute of
    /// [input](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#attr-checked).
    /// It exists to override standard behavior of `checked` attribute, because
    /// in original HTML it sets `defaultChecked` value of `InputElement`, but for reactive
    /// frameworks it's more useful to control `checked` value of an `InputElement`.
    pub checked: bool,
    /// A node reference used for DOM access in Component lifecycle methods
    pub node_ref: NodeRef,
    /// Keeps handler for attached listeners to have an opportunity to drop them later.
    captured: Vec<EventListener>,

    pub key: Option<String>,
}

impl Clone for VTag {
    fn clone(&self) -> Self {
        VTag {
            tag: self.tag.clone(),
            element_type: self.element_type,
            reference: None,
            listeners: self.listeners.clone(),
            attributes: self.attributes.clone(),
            children: self.children.clone(),
            value: self.value.clone(),
            kind: self.kind.clone(),
            checked: self.checked,
            node_ref: self.node_ref.clone(),
            key: self.key.clone(),
            captured: Vec::new(),
        }
    }
}

impl VTag {
    /// Creates a new `VTag` instance with `tag` name (cannot be changed later in DOM).
    pub fn new<S: Into<Cow<'static, str>>>(tag: S) -> Self {
        let tag: Cow<'static, str> = tag.into();
        let element_type = ElementType::from_tag(&tag);
        VTag {
            tag,
            element_type,
            reference: None,
            attributes: Attributes::new(),
            listeners: Vec::new(),
            captured: Vec::new(),
            children: VList::new(),
            node_ref: NodeRef::default(),
            key: None,
            value: None,
            kind: None,
            // In HTML node `checked` attribute sets `defaultChecked` parameter,
            // but we use own field to control real `checked` parameter
            checked: false,
        }
    }

    /// Returns tag of an `Element`. In HTML tags are always uppercase.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Add `VNode` child.
    pub fn add_child(&mut self, child: VNode) {
        self.children.add_child(child);
    }

    /// Add multiple `VNode` children.
    pub fn add_children(&mut self, children: impl IntoIterator<Item = VNode>) {
        self.children.add_children(children);
    }

    /// Sets `value` for an
    /// [InputElement](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input).
    pub fn set_value<T: ToString>(&mut self, value: &T) {
        self.value = Some(value.to_string());
    }

    /// Sets `kind` property of an
    /// [InputElement](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input).
    /// Same as set `type` attribute.
    pub fn set_kind<T: ToString>(&mut self, value: &T) {
        self.kind = Some(value.to_string());
    }

    /// Sets `checked` property of an
    /// [InputElement](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input).
    /// (Not a value of node's attribute).
    pub fn set_checked(&mut self, value: bool) {
        self.checked = value;
    }

    /// Adds attribute to a virtual node. Not every attribute works when
    /// it set as attribute. We use workarounds for:
    /// `type/kind`, `value` and `checked`.
    ///
    /// If this virtual node has this attribute present, the value is replaced.
    pub fn add_attribute<T: ToString>(&mut self, name: &str, value: &T) {
        self.attributes.insert(name.to_owned(), value.to_string());
    }

    /// Adds attributes to a virtual node. Not every attribute works when
    /// it set as attribute. We use workarounds for:
    /// `type/kind`, `value` and `checked`.
    pub fn add_attributes(&mut self, attrs: Vec<(String, String)>) {
        for (name, value) in attrs {
            self.attributes.insert(name, value);
        }
    }

    /// Adds new listener to the node.
    /// It's boxed because we want to keep it in a single list.
    /// Later `Listener::attach` will attach an actual listener to a DOM node.
    pub fn add_listener(&mut self, listener: Rc<dyn Listener>) {
        self.listeners.push(listener);
    }

    /// Adds new listeners to the node.
    /// They are boxed because we want to keep them in a single list.
    /// Later `Listener::attach` will attach an actual listener to a DOM node.
    pub fn add_listeners(&mut self, listeners: Vec<Rc<dyn Listener>>) {
        for listener in listeners {
            self.listeners.push(listener);
        }
    }

    /// Every render it removes all listeners and attach it back later
    /// TODO(#943): Compare references of handler to do listeners update better
    fn recreate_listeners(&mut self, ancestor: &mut Option<Box<Self>>) {
        if let Some(ancestor) = ancestor.as_mut() {
            ancestor.captured.clear();
        }

        let element = self.reference.clone().expect("element expected");

        for listener in self.listeners.drain(..) {
            let handle = listener.attach(&element);
            self.captured.push(handle);
        }
    }

    fn refresh_value(&mut self) {
        if let Some(element) = self.reference.as_ref() {
            if self.element_type == ElementType::Input {
                let input_el = element.dyn_ref::<InputElement>();
                if let Some(input) = input_el {
                    let current_value = input.value();
                    self.set_value(&current_value)
                }
            } else if self.element_type == ElementType::Textarea {
                let textarea_el = element.dyn_ref::<TextAreaElement>();
                if let Some(tae) = textarea_el {
                    let current_value = &tae.value();
                    self.set_value(&current_value)
                }
            }
        }
    }

    /// This handles patching of attributes when the keys are equal but
    /// the values are different.
    fn diff_attributes<'a>(
        &'a self,
        ancestor: &'a Option<Box<Self>>,
    ) -> impl Iterator<Item = Patch<&'a str, &'a str>> + 'a {
        // Only change what is necessary.
        let to_add_or_replace =
            self.attributes.iter().filter_map(move |(key, value)| {
                match ancestor
                    .as_ref()
                    .and_then(|ancestor| ancestor.attributes.get(&**key))
                {
                    None => Some(Patch::Add(&**key, &**value)),
                    Some(ancestor_value) if value != ancestor_value => {
                        Some(Patch::Replace(&**key, &**value))
                    }
                    _ => None,
                }
            });
        let to_remove = ancestor
            .iter()
            .flat_map(|ancestor| ancestor.attributes.keys())
            .filter(move |key| !self.attributes.contains_key(&**key))
            .map(|key| Patch::Remove(&**key));

        to_add_or_replace.chain(to_remove)
    }

    /// Similar to `diff_attributers` except there is only a single `kind`.
    fn diff_kind<'a>(&'a self, ancestor: &'a Option<Box<Self>>) -> Option<Patch<&'a str, ()>> {
        match (
            self.kind.as_ref(),
            ancestor.as_ref().and_then(|anc| anc.kind.as_ref()),
        ) {
            (Some(ref left), Some(ref right)) => {
                if left != right {
                    Some(Patch::Replace(&**left, ()))
                } else {
                    None
                }
            }
            (Some(ref left), None) => Some(Patch::Add(&**left, ())),
            (None, Some(right)) => Some(Patch::Remove(&**right)),
            (None, None) => None,
        }
    }

    /// Almost identical in spirit to `diff_kind`
    fn diff_value<'a>(&'a self, ancestor: &'a Option<Box<Self>>) -> Option<Patch<&'a str, ()>> {
        match (
            self.value.as_ref(),
            ancestor.as_ref().and_then(|anc| anc.value.as_ref()),
        ) {
            (Some(ref left), Some(ref right)) => {
                if left != right {
                    Some(Patch::Replace(&**left, ()))
                } else {
                    None
                }
            }
            (Some(ref left), None) => Some(Patch::Add(&**left, ())),
            (None, Some(right)) => Some(Patch::Remove(&**right)),
            (None, None) => None,
        }
    }

    fn apply_diffs(&mut self, ancestor: &Option<Box<Self>>) {
        let element = self.reference.as_ref().expect("element expected");

        // Update parameters
        let changes = self.diff_attributes(ancestor);

        // apply attribute patches including an optional "class"-attribute patch
        for change in changes {
            match change {
                Patch::Add(key, value) | Patch::Replace(key, value) => {
                    element
                        .set_attribute(&key, &value)
                        .expect("invalid attribute key");
                }
                Patch::Remove(key) => {
                    element.remove_attribute(&key).expect("could not remove attribute");
                }
            }
        }

        if self.element_type == ElementType::Button {
            if let Some(button) = element.dyn_ref::<HtmlButtonElement>() {
                if let Some(change) = self.diff_kind(ancestor) {
                    let kind = match change {
                        Patch::Add(kind, _) | Patch::Replace(kind, _) => kind,
                        Patch::Remove(_) => "",
                    };
                    button.set_type(kind);
                }
            }
        }

        // `input` element has extra parameters to control
        // I override behavior of attributes to make it more clear
        // and useful in templates. For example I interpret `checked`
        // attribute as `checked` parameter, not `defaultChecked` as browsers do
        if self.element_type == ElementType::Input {
            if let Some(input) = element.dyn_ref::<InputElement>() {

                if let Some(change) = self.diff_kind(ancestor) {
                    let kind = match change {
                        Patch::Add(kind, _) | Patch::Replace(kind, _) => kind,
                        Patch::Remove(_) => "",
                    };
                    input.set_type(kind)
                }

                if let Some(change) = self.diff_value(ancestor) {
                    let raw_value = match change {
                        Patch::Add(kind, _) | Patch::Replace(kind, _) => kind,
                        Patch::Remove(_) => "",
                    };
                    input.set_value(raw_value);
                }

                // IMPORTANT! This parameter has to be set every time
                // to prevent strange behaviour in the browser when the DOM changes
                set_checked(&input, self.checked);
            }
        } else if self.element_type == ElementType::Textarea {
            if let Some(tae) = element.dyn_ref::<TextAreaElement>() {
                if let Some(change) = self.diff_value(ancestor) {
                    let value = match change {
                        Patch::Add(kind, _) | Patch::Replace(kind, _) => kind,
                        Patch::Remove(_) => "",
                    };
                    tae.set_value(value);
                }
            }
        }
    }

    fn create_element(&self, parent: &Element) -> Element {
        if self.tag == "svg"
            || parent
                .namespace_uri()
                .map_or(false, |ns| ns == SVG_NAMESPACE)
        {
            let namespace = SVG_NAMESPACE;
            document()
                .create_element_ns(Some(namespace), &self.tag)
                .expect("can't create namespaced element for vtag")
        } else {
            document()
                .create_element(&self.tag)
                .expect("can't create element for vtag")
        }
    }
}

impl VDiff for VTag {
    /// Remove VTag from parent.
    fn detach(&mut self, parent: &Element) {
        let node = self
            .reference
            .take()
            .expect("tried to remove not rendered VTag from DOM");

        // recursively remove its children
        self.children.detach(&node);
        if parent.remove_child(&node).is_err() {
            warn!("Node not found to remove VTag");
        }
    }

    /// Renders virtual tag over DOM `Element`, but it also compares this with an ancestor `VTag`
    /// to compute what to patch in the actual DOM nodes.
    fn apply(
        &mut self,
        parent_scope: &AnyScope,
        parent: &Element,
        next_sibling: NodeRef,
        ancestor: Option<VNode>,
    ) -> NodeRef {
        let mut ancestor_tag = ancestor.and_then(|mut ancestor| {
            match ancestor {
                // If the ancestor is a tag of the same type, don't recreate, keep the
                // old tag and update its attributes and children.
                VNode::VTag(vtag) if self.tag == vtag.tag => Some(vtag),
                _ => {
                    let element = self.create_element(parent);
                    super::insert_node(&element, parent, Some(ancestor.first_node()));
                    self.reference = Some(element);
                    ancestor.detach(parent);
                    None
                }
            }
        });

        if let Some(ref mut ancestor_tag) = &mut ancestor_tag {
            // Refresh the current value to later compare it against the desired value
            // since it may have been changed since we last set it.
            ancestor_tag.refresh_value();
            // Preserve the reference that already exists.
            self.reference = ancestor_tag.reference.take();
        } else if self.reference.is_none() {
            let element = self.create_element(parent);
            super::insert_node(&element, parent, next_sibling.get());
            self.reference = Some(element);
        }

        self.apply_diffs(&ancestor_tag);
        self.recreate_listeners(&mut ancestor_tag);

        // Process children
        let element = self.reference.as_ref().expect("Reference should be set");
        if !self.children.is_empty() {
            self.children.apply(
                parent_scope,
                element,
                NodeRef::default(),
                ancestor_tag.map(|a| a.children.into()),
            );
        } else if let Some(mut ancestor_tag) = ancestor_tag {
            ancestor_tag.children.detach(element);
        }

        let node = element.deref();
        self.node_ref.set(Some(node.clone()));
        self.node_ref.clone()
    }
}

/// Set `checked` value for the `InputElement`.
fn set_checked(input: &InputElement, value: bool) {
    input.set_checked(value)
}

impl PartialEq for VTag {
    fn eq(&self, other: &VTag) -> bool {
        self.tag == other.tag
            && self.value == other.value
            && self.kind == other.kind
            && self.checked == other.checked
            && self.listeners.len() == other.listeners.len()
            && self
                .listeners
                .iter()
                .map(|l| l.kind())
                .eq(other.listeners.iter().map(|l| l.kind()))
            && self.attributes == other.attributes
            && self.children == other.children
    }
}

impl<T> Transformer<T, T> for VTag {
    fn transform(from: T) -> T {
        from
    }
}

impl<'a, T> Transformer<&'a T, T> for VTag
where
    T: Clone,
{
    fn transform(from: &'a T) -> T {
        from.clone()
    }
}
