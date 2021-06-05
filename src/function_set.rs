#![allow(unused)]

use slotmap::Key;

use super::{
    iterators::{EdgeFaces, VertexEdges, VertexFaces, VertexInHalfEdges, VertexOutHalfEdges, FaceVertices, FaceEdges, FaceFaces, VertexVertex},
    Data, EdgeHandle, FaceHandle, HalfEdgeHandle, HalfEdgeGraph, VertexHandle,
};

macro_rules! impl_fn {
    (
        struct $name:ident ($handle:ty, $data:ident, $map:ident) {
            $($props:tt)+
        }
    ) => {
        pub struct $name<'mesh, DataTypes: Data> {
            mesh: &'mesh HalfEdgeGraph<DataTypes>,
            handle: $handle,
        }
        impl<'mesh, DataTypes: Data> Clone for $name<'mesh, DataTypes> {
            fn clone(&self) -> Self {
                Self {
                    mesh: self.mesh,
                    handle: self.handle
                }
            }
        }
        impl<'mesh, DataTypes: Data> Copy for $name<'mesh, DataTypes> {}

        impl<'mesh, DataTypes: Data> std::cmp::PartialEq<Self> for $name<'mesh, DataTypes> {
            fn eq(&self, other: &Self) -> bool {
                self.handle == other.handle
            }
        }
        impl<'mesh, DataTypes: Data> std::cmp::PartialEq<$handle> for $name<'mesh, DataTypes> {
            fn eq(&self, other: &$handle) -> bool {
                self.handle == *other
            }
        }
        impl<'mesh, DataTypes: Data> std::cmp::PartialEq<$name<'mesh, DataTypes>> for $handle {
            fn eq(&self, other: &$name<'mesh, DataTypes>) -> bool {
                self == &other.handle
            }
        }


        impl<'mesh, DataTypes: Data> std::ops::Deref for $name<'mesh, DataTypes> {
            type Target = DataTypes::$data;

            fn deref(&self) -> &Self::Target {
                $name::data(self)
            }
        }

        impl<'mesh, DataTypes: Data> $name<'mesh, DataTypes> {
            pub fn new(mesh: &'mesh HalfEdgeGraph<DataTypes>, handle: $handle) -> Self {
                debug_assert!(mesh.$map.contains_key(handle));

                Self {
                    mesh,
                    handle,
                }
            }

            pub fn handle(&self) -> $handle {
                self.handle
            }

            pub fn mesh(&self) -> &'mesh HalfEdgeGraph<DataTypes> {
                self.mesh
            }

            pub fn data(&self) -> &DataTypes::$data {
                &self.mesh.$map[self.handle].data
            }

            impl_fn! {
                __props ($map) {
                    $($props)+
                }
            }
        }

        impl<'mesh, DataTypes: Data> std::fmt::Debug for $name<'mesh, DataTypes> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.handle.fmt(f)
            }
        }
    };

    (
        __props ($map:ident) {}
    ) => {};

    (
        __props ($map:ident) {
            $prop:ident : $type:ident,
            $($rest:tt)*
        }
    ) => {
        pub fn $prop(&self) -> $type<'mesh, DataTypes> {
            let info = &self.mesh.$map[self.handle];
            let prop = info.$prop;

            $type::new(self.mesh, prop)
        }

        impl_fn! {
            __props ($map) {
                $($rest)*
            }
        }
    };

    (
        __props ($map:ident) {
            $prop:ident : Option<$type:ident>,
            $($rest:tt)*
        }
    ) => {
        pub fn $prop(&self) -> Option<$type<'mesh, DataTypes>> {
            let info = &self.mesh.$map[self.handle];
            let prop = info.$prop;

            if prop.is_null() {
                None
            }
            else {
                Some($type::new(self.mesh, prop))
            }
        }

        impl_fn! {
            __props ($map) {
                $($rest)*
            }
        }
    };

    (
        __props ($map:ident) {
            $prop:ident -> $iter:ident,
            $($rest:tt)*
        }
    ) => {
        pub fn $prop(&self) -> $iter<DataTypes> {
            $iter::new(*self)
        }

        impl_fn! {
            __props ($map) {
                $($rest)*
            }
        }
    };    (

    __props ($map:ident) {
            pub fn $prop:ident(&$self:ident) -> $result:ty {
                $($fn:tt)+
            }
            $($rest:tt)*
        }
    ) => {
        pub fn $prop(&$self) -> $result {
            $($fn)+
        }

        impl_fn! {
            __props ($map) {
                $($rest)*
            }
        }
    };
}

impl_fn!(
    struct HalfEdgeFn (HalfEdgeHandle, HalfEdge, half_edges) {
        pair: HalfEdgeFn,
        next: HalfEdgeFn,
        prev: HalfEdgeFn,

        face: Option<FaceFn>,
        vertex: VertexFn,
        edge: EdgeFn,
    }
);

impl_fn!(
    struct FaceFn (FaceHandle, Face, faces) {
        hedge: HalfEdgeFn,

        vertices -> FaceVertices,
        edges -> FaceEdges,
        faces -> FaceFaces,
    }
);

impl_fn!(
    struct VertexFn (VertexHandle, Vertex, vertices) {
        hedge: Option<HalfEdgeFn>,

        in_half_edges -> VertexInHalfEdges,
        out_half_edges -> VertexOutHalfEdges,
        edges -> VertexEdges,
        faces -> VertexFaces,
        vertices -> VertexVertex,
    }

);

impl_fn!(
    struct EdgeFn (EdgeHandle, Edge, edges) {
        hedge: HalfEdgeFn,

        pub fn vertices(&self) -> [VertexFn<'mesh, DataTypes>; 2]  {
            let hedge = self.hedge();
            let pair = hedge.pair();
            [hedge.vertex(), pair.vertex()]
        }

        faces -> EdgeFaces,
    }
);
