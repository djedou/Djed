use super::{Transformer, VDiff};
use super::v_node::VNode;
use crate::djed::{AnyScope, Component, ComponentUpdate, NodeRef, Scope, Scoped};
use crate::utils::document;
use std::any::TypeId;
use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use web_sys::{Element, Node};


/// A virtual component.
pub struct VComp {
    type_id: TypeId,
    scope: Option<Box<dyn Scoped>>,
    props: Option<Box<dyn Mountable>>,
    pub(crate) node_ref: NodeRef,
    pub(crate) key: Option<String>,
}

impl Clone for VComp {
    fn clone(&self) -> Self {
        if self.scope.is_some() {
            panic!("Mounted components are not allowed to be cloned! {}",2);
        }

        Self {
            type_id: self.type_id,
            scope: None,
            props: self.props.as_ref().map(|m| m.copy()),
            node_ref: self.node_ref.clone(),
            key: self.key.clone(),
        }
    }
}

/// A virtual child component.
pub struct VChild<COMP: Component> {
    /// The component properties
    pub props: COMP::Props,
    /// Reference to the mounted node
    node_ref: NodeRef,
    key: Option<String>,
}

impl<COMP: Component> Clone for VChild<COMP> {
    fn clone(&self) -> Self {
        VChild {
            props: self.props.clone(),
            node_ref: self.node_ref.clone(),
            key: self.key.clone(),
        }
    }
}

impl<COMP: Component> PartialEq for VChild<COMP>
where
    COMP::Props: PartialEq,
{
    fn eq(&self, other: &VChild<COMP>) -> bool {
        self.props == other.props
    }
}

impl<COMP> VChild<COMP>
where
    COMP: Component,
{
    /// Creates a child component that can be accessed and modified by its parent.
    pub fn new(props: COMP::Props, node_ref: NodeRef, key: Option<String>) -> Self {
        Self {
            props,
            node_ref,
            key,
        }
    }
}

impl<COMP> From<VChild<COMP>> for VComp
where
    COMP: Component,
{
    fn from(vchild: VChild<COMP>) -> Self {
        VComp::new::<COMP>(vchild.props, vchild.node_ref, vchild.key)
    }
}

impl VComp {
    /// Creates a new `VComp` instance.
    pub fn new<COMP>(props: COMP::Props, node_ref: NodeRef, key: Option<String>) -> Self
    where
        COMP: Component,
    {
        VComp {
            type_id: TypeId::of::<COMP>(),
            node_ref,
            props: Some(Box::new(PropsWrapper::<COMP>::new(props))),
            scope: None,
            key,
        }
    }

    #[allow(unused)]
    pub(crate) fn root_vnode(&self) -> Option<impl Deref<Target = VNode> + '_> {
        self.scope.as_ref().and_then(|scope| scope.root_vnode())
    }
}

trait Mountable {
    fn copy(&self) -> Box<dyn Mountable>;
    fn mount(
        self: Box<Self>,
        node_ref: NodeRef,
        parent_scope: &AnyScope,
        parent: Element,
        next_sibling: NodeRef,
    ) -> Box<dyn Scoped>;
    fn reuse(self: Box<Self>, scope: &dyn Scoped, next_sibling: NodeRef);
}

struct PropsWrapper<COMP: Component> {
    props: COMP::Props,
}

impl<COMP: Component> PropsWrapper<COMP> {
    pub fn new(props: COMP::Props) -> Self {
        Self { props }
    }
}

impl<COMP: Component> Mountable for PropsWrapper<COMP> {
    fn copy(&self) -> Box<dyn Mountable> {
        let wrapper: PropsWrapper<COMP> = PropsWrapper {
            props: self.props.clone(),
        };
        Box::new(wrapper)
    }

    fn mount(
        self: Box<Self>,
        node_ref: NodeRef,
        parent_scope: &AnyScope,
        parent: Element,
        next_sibling: NodeRef,
    ) -> Box<dyn Scoped> {
        let scope: Scope<COMP> = Scope::new(Some(parent_scope.clone()));
        let scope = scope.mount_in_place(
            parent,
            next_sibling,
            Some(VNode::VRef(node_ref.get().unwrap())),
            node_ref,
            self.props,
        );

        Box::new(scope)
    }

    fn reuse(self: Box<Self>, scope: &dyn Scoped, next_sibling: NodeRef) {
        let scope: Scope<COMP> = scope.to_any().downcast();
        scope.update(ComponentUpdate::Properties(self.props, next_sibling), false);
    }
}

impl VDiff for VComp {
    fn detach(&mut self, _parent: &Element) {
        self.scope.take().expect("VComp is not mounted").destroy();
    }

    fn apply(
        &mut self,
        parent_scope: &AnyScope,
        parent: &Element,
        next_sibling: NodeRef,
        ancestor: Option<VNode>,
    ) -> NodeRef {
        let mountable = self.props.take().expect("VComp has already been mounted");

        if let Some(mut ancestor) = ancestor {
            if let VNode::VComp(ref mut vcomp) = &mut ancestor {
                // If the ancestor is the same type, reuse it and update its properties
                if self.type_id == vcomp.type_id {
                    self.node_ref.link(vcomp.node_ref.clone());
                    let scope = vcomp.scope.take().expect("VComp is not mounted");
                    mountable.reuse(scope.borrow(), next_sibling);
                    self.scope = Some(scope);
                    return vcomp.node_ref.clone();
                }
            }

            ancestor.detach(parent);
        }

        let placeholder: Node = document().create_text_node("").into();
        super::insert_node(&placeholder, parent, next_sibling.get());
        self.node_ref.set(Some(placeholder));
        let scope = mountable.mount(
            self.node_ref.clone(),
            parent_scope,
            parent.to_owned(),
            next_sibling,
        );
        self.scope = Some(scope);
        self.node_ref.clone()
    }
}

impl<T> Transformer<T, T> for VComp {
    fn transform(from: T) -> T {
        from
    }
}

impl<'a, T> Transformer<&'a T, T> for VComp
where
    T: Clone,
{
    fn transform(from: &'a T) -> T {
        from.clone()
    }
}

impl<'a> Transformer<&'a str, String> for VComp {
    fn transform(from: &'a str) -> String {
        from.to_owned()
    }
}

impl<T> Transformer<T, Option<T>> for VComp {
    fn transform(from: T) -> Option<T> {
        Some(from)
    }
}

impl<'a, T> Transformer<&'a T, Option<T>> for VComp
where
    T: Clone,
{
    fn transform(from: &T) -> Option<T> {
        Some(from.clone())
    }
}

impl<'a> Transformer<&'a str, Option<String>> for VComp {
    fn transform(from: &'a str) -> Option<String> {
        Some(from.to_owned())
    }
}

impl<'a> Transformer<Option<&'a str>, Option<String>> for VComp {
    fn transform(from: Option<&'a str>) -> Option<String> {
        from.map(|s| s.to_owned())
    }
}

impl PartialEq for VComp {
    fn eq(&self, other: &VComp) -> bool {
        self.type_id == other.type_id
    }
}

impl fmt::Debug for VComp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VComp")
    }
}

impl<COMP: Component> fmt::Debug for VChild<COMP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VChild<_>")
    }
}
