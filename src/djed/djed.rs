
use crate::djed_dom::{VChild, VNode};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::Node;
use super::Scope;


/// This type indicates that component should be rendered again.
pub type ShouldRender = bool;

/// Components are the basic building blocks of the UI in a djed app. Each Component
/// chooses how to display itself using received props and self-managed state.
/// Components can be dynamic and interactive by declaring messages that are
/// triggered and handled asynchronously. This async update mechanism is inspired by
/// Elm and the actor model used in the Actix framework.
pub trait Component: Sized + 'static {
    /// Messages are used to make Components dynamic and interactive. Simple
    /// Component's can declare their Message type to be `()`. Complex Component's
    /// commonly use an enum to declare multiple Message types.
    type State: 'static;

    /// Properties are the inputs to a Component and should not mutated within a
    /// Component. They are passed to a Component using a JSX-style syntax.
    /// ```
    ///# use djed::{Html, Component, Properties, ComponentLink, html};
    ///# struct Model;
    ///# #[derive(Clone, Properties)]
    ///# struct Props {
    ///#     prop: String,
    ///# }
    ///# impl Component for Model {
    ///#     type State = ();
    ///#     type Props = Props;
    ///#     fn create(props: Self::Props,link: ComponentLink<Self>) -> Self {unimplemented!()}
    ///#     fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
    ///#     fn change(&mut self, _: Self::Properties) -> bool {unimplemented!()}
    ///#     fn view(&self) -> Html {
    /// html! {
    ///     <Model prop="value" />
    /// }
    ///# }}
    /// ```
    type Props: Props;

    /// Components are created with their properties as well as a `ComponentLink` which
    /// can be used to send messages and create callbacks for triggering updates.
    fn create(props: Self::Props, link: ComponentLink<Self>) -> Self;

    /// Components handle messages in their `update` method and commonly use this method
    /// to update their state and (optionally) re-render themselves.
    fn update(&mut self, state: Self::State) -> ShouldRender;

    /// When the parent of a Component is re-rendered, it will either be re-created or
    /// receive new properties in the `change` lifecycle method. Component's can choose
    /// to re-render if the new properties are different than the previously
    /// received properties. Most Component's will use props with a `PartialEq`
    /// impl and will be implemented like this:
    /// ```
    ///# use djed::{Html, Component, ComponentLink, html, ShouldRender};
    ///# struct Model{props: ()};
    ///# impl Component for Model {
    ///#     type Message = ();type Properties = ();
    ///#     fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
    ///#     fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
    ///#     fn view(&self) -> Html {unimplemented!()}
    /// fn change(&mut self, props: Self::Properties) -> ShouldRender {
    ///     if self.props != props {
    ///         self.props = props;
    ///         true
    ///     } else {
    ///         false
    ///     }
    /// }
    ///# }
    /// ```
    /// Components which don't have properties should always return false.
    fn change(&mut self, _props: Self::Props) -> ShouldRender;

    /// Components define their visual layout using a JSX-style syntax through the use of the
    /// `html!` procedural macro. The full guide to using the macro can be found in [djed's
    /// documentation](https://djed.rs/docs/concepts/html).
    fn view(&self) -> Html;

    /// The `rendered` method is called after each time a Component is rendered but
    /// before the browser updates the page.
    /// ## Examples
    /// ```rust
    ///# use djed::{Html, Component, ComponentLink, html, ShouldRender};
    ///# struct Model{props: ()};
    ///# impl Model { fn setup_element(&self) { } }
    ///# impl Component for Model {
    ///#     type State = ();
    ///#     type Props = ();
    ///#     fn create(props: Self::Props,link: ComponentLink<Self>) -> Self {unimplemented!()}
    ///#     fn update(&mut self,msg: Self::State) -> bool {unimplemented!()}
    ///#     fn view(&self) -> Html {unimplemented!()}
    ///#     fn change(&mut self, _props: Self::Props) -> ShouldRender { unimplemented!() }
    /// fn rendered(&mut self, first_render: bool) {
    ///    if first_render {
    ///      self.setup_element(); // Similar to 'mounted' in other frameworks
    ///    }
    /// }
    ///# }
    fn rendered(&mut self, _first_render: bool) {}
}

/// A type which expected as a result of `view` function implementation.
pub type Html = VNode;

/// A type used for accepting children elements in Component::Properties.
///
/// # Example
/// **`model.rs`**
///
/// In this example, the `Wrapper` component is used to wrap other elements.
/// ```
///# use djed::{Children, Html, Properties, Component, ComponentLink, html};
///# #[derive(Clone, Properties)]
///# struct WrapperProps {
///#     children: Children,
///# }
///# struct Wrapper;
///# impl Component for Wrapper{
///#     type State = ();
///#     type Props = WrapperProps;
///#     fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
///#     fn update(&mut self,state: Self::State) -> bool {unimplemented!()}
///#     fn change(&mut self, _: Self::Props) -> bool {unimplemented!()}
///#     // This is not a valid implementation.  This is done for space convenience.
///#     fn view(&self) -> Html {
/// html! {
///     <Wrapper>
///         <h4>{ "Hi" }</h4>
///         <div>{ "Hello" }</div>
///     </Wrapper>
/// }
///#     }
///# }
/// ```
///
/// **`wrapper.rs`**
///
/// The Wrapper component must define a `children` property in order to wrap other elements. The
/// children property can be used to render the wrapped elements.
/// ```
///# use djed::{Children, Html, Properties, Component, ComponentLink, html};
/// #[derive(Clone, Properties)]
/// struct WrapperProps {
///     children: Children,
/// }
///
///# struct Wrapper {props: WrapperProps};
/// impl Component for Wrapper {
///     // ...
///#     type Message = ();
///#     type Properties = WrapperProps;
///#     fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
///#     fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
///#     fn change(&mut self, _: Self::Properties) -> bool {unimplemented!()}
///     fn view(&self) -> Html {
///         html! {
///             <div id="container">
///                 { self.props.children.clone() }
///             </div>
///         }
///     }
/// }
/// ```
pub type Children = ChildrenRenderer<Html>;

/// A type used for accepting children elements in Component::Properties and accessing their props.
///
/// # Example
/// **`model.rs`**
///
/// In this example, the `List` component can wrap `ListItem` components.
/// ```
///# use djed::{html, Component, Renderable, Html, ComponentLink, ChildrenWithProps, Properties};
///#
///# #[derive(Clone, Properties)]
///# struct ListProps {
///#     children: ChildrenWithProps<ListItem>,
///# }
///# struct List;
///# impl Component for List {
///#     type State = ();
///#     type Properties = ListProps;
///#     fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {unimplemented!()}
///#     fn update(&mut self, msg: Self::Message) -> bool {unimplemented!()}
///#     fn change(&mut self, _: Self::Properties) -> bool {unimplemented!()}
///#     fn view(&self) -> Html {unimplemented!()}
///# }
///# #[derive(Clone, Properties)]
///# struct ListItemProps {
///#     value: String
///# }
///# struct ListItem;
///# impl Component for ListItem {
///#     type Message = ();
///#     type Properties = ListItemProps;
///#     fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {unimplemented!()}
///#     fn update(&mut self, msg: Self::Message) -> bool {unimplemented!()}
///#     fn change(&mut self, _: Self::Properties) -> bool {unimplemented!()}
///#     fn view(&self) -> Html {unimplemented!()}
///# }
///# fn view() -> Html {
/// html!{
///   <List>
///     <ListItem value="a" />
///     <ListItem value="b" />
///     <ListItem value="c" />
///   </List>
/// }
///# }
/// ```
///
/// **`list.rs`**
///
/// The `List` component must define a `children` property in order to wrap the list items. The
/// `children` property can be used to filter, mutate, and render the items.
/// ```
///# use djed::{html, Component, Html, ChildrenWithProps, ComponentLink, Properties};
///#
/// #[derive(Clone, Properties)]
/// struct ListProps {
///     children: ChildrenWithProps<ListItem>,
/// }
///
///# struct List {props: ListProps};
/// impl Component for List {
///#     type Message = ();
///#     type Properties = ListProps;
///#     fn create(props: Self::Properties,link: ComponentLink<Self>) -> Self {unimplemented!()}
///#     fn update(&mut self,msg: Self::Message) -> bool {unimplemented!()}
///#     fn change(&mut self, _: Self::Properties) -> bool {unimplemented!()}
///     // ...
///     fn view(&self) -> Html {
///         html!{{
///             for self.props.children.iter().map(|mut item| {
///                 item.props.value = format!("item-{}", item.props.value);
///                 item
///             })
///         }}
///     }
/// }
///#
///# #[derive(Clone, Properties)]
///# struct ListItemProps {
///#     #[prop_or_default]
///#     value: String
///# }
///#
///# struct ListItem;
///# impl Component for ListItem {
///#     type Message = ();
///#     type Properties = ListItemProps;
///#     fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {unimplemented!()}
///#     fn update(&mut self, msg: Self::Message) -> bool {unimplemented!()}
///#     fn change(&mut self, _: Self::Properties) -> bool {unimplemented!()}
///#     fn view(&self) -> Html {unimplemented!()}
///# }
/// ```
pub type ChildrenWithProps<CHILD> = ChildrenRenderer<VChild<CHILD>>;

/// A type used for rendering children html.
#[derive(Clone)]
pub struct ChildrenRenderer<T> {
    children: Vec<T>,
}

impl<T: PartialEq> PartialEq for ChildrenRenderer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
    }
}

impl<T> ChildrenRenderer<T>
where
    T: Clone + Into<VNode>,
{
    /// Create children
    pub fn new(children: Vec<T>) -> Self {
        Self { children }
    }

    /// Children list is empty
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Number of children elements
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Render children components and return `Iterator`
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a {
        // clone each child lazily.
        // This way `self.iter().next()` only has to clone a single node.
        self.children.iter().cloned()
    }
}

impl<T> Default for ChildrenRenderer<T> {
    fn default() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl<T> fmt::Debug for ChildrenRenderer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ChildrenRenderer<_>")
    }
}

impl<T> IntoIterator for ChildrenRenderer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

/// Wrapped Node reference for later use in Component lifecycle methods.
///
/// # Example
/// Focus an `<input>` element on mount.
/// ```
/// #[cfg(feature = "std_web")]
/// use stdweb::web::{html_element::InputElement, IHtmlElement};
/// #[cfg(feature = "web_sys")]
/// use web_sys::HtmlInputElement as InputElement;
///# use djed::prelude::*;
///
/// pub struct Input {
///     node_ref: NodeRef,
/// }
///
/// impl Component for Input {
///     type Message = ();
///     type Properties = ();
///
///     fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
///         Input {
///             node_ref: NodeRef::default(),
///         }
///     }
///
///     fn rendered(&mut self, first_render: bool) {
///         if first_render {
///             if let Some(input) = self.node_ref.cast::<InputElement>() {
///                 input.focus();
///             }
///         }
///     }
///
///     fn change(&mut self, _: Self::Properties) -> ShouldRender {
///         false
///     }
///
///     fn update(&mut self, _: Self::Message) -> ShouldRender {
///         false
///     }
///
///     fn view(&self) -> Html {
///         html! {
///             <input ref=self.node_ref.clone() type="text" />
///         }
///     }
/// }
#[derive(Debug, Default, Clone)]
pub struct NodeRef(Rc<RefCell<NodeRefInner>>);

impl PartialEq for NodeRef {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr() || Some(self) == other.0.borrow().link.as_ref()
    }
}

#[derive(PartialEq, Debug, Default, Clone)]
struct NodeRefInner {
    node: Option<Node>,
    link: Option<NodeRef>,
}

impl NodeRef {
    /// Get the wrapped Node reference if it exists
    pub fn get(&self) -> Option<Node> {
        let inner = self.0.borrow();
        inner.node.clone().or_else(|| inner.link.as_ref()?.get())
    }

    /// Try converting the node reference into another form
    pub fn cast<INTO: AsRef<Node> + From<JsValue>>(
        &self,
    ) -> Option<INTO> {
        let node = self.get();
        node.map(Into::into).map(INTO::from)
    }

    /// Wrap an existing `Node` in a `NodeRef`
    pub(crate) fn new(node: Node) -> Self {
        let node_ref = NodeRef::default();
        node_ref.set(Some(node));
        node_ref
    }

    /// Place a Node in a reference for later use
    pub(crate) fn set(&self, node: Option<Node>) {
        let mut this = self.0.borrow_mut();
        this.node = node;
        this.link = None;
    }

    /// Link a downstream `NodeRef`
    pub(crate) fn link(&self, node_ref: Self) {
        // Avoid circular references
        if self == &node_ref {
            return;
        }

        let mut this = self.0.borrow_mut();
        this.node = None;
        this.link = Some(node_ref);
    }
}

/// Trait for rendering virtual DOM elements
pub trait Renderable {
    /// Called by rendering loop.
    fn render(&self) -> Html;
}

impl<COMP: Component> Renderable for COMP {
    fn render(&self) -> Html {
        self.view()
    }
}

/// Trait for building properties for a component
pub trait Props: Clone {
    /// Builder that will be used to construct properties
    type Builder;

    /// Entrypoint for building properties
    fn builder() -> Self::Builder;
}

/// Builder for when a component has no properties
#[derive(Debug)]
#[doc(hidden)]
pub struct EmptyBuilder;

impl Props for () {
    type Builder = EmptyBuilder;

    fn builder() -> Self::Builder {
        EmptyBuilder
    }
}

impl EmptyBuilder {
    /// Build empty properties
    pub fn build(self) {}
}

/// Link to component's scope for creating callbacks.
pub type ComponentLink<COMP> = Scope<COMP>;

/// A bridging type for checking `href` attribute value.
#[derive(Debug)]
pub struct Href {
    link: String,
}

impl From<String> for Href {
    fn from(link: String) -> Self {
        Href { link }
    }
}

impl<'a> From<&'a str> for Href {
    fn from(link: &'a str) -> Self {
        Href {
            link: link.to_owned(),
        }
    }
}

impl ToString for Href {
    fn to_string(&self) -> String {
        self.link.to_owned()
    }
}
