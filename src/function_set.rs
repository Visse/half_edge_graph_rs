use crate::{
    Data, Edge, EdgeFaces, EdgeFacesMut, EdgeHandle, Face, FaceEdges, FaceEdgesMut, FaceFaces,
    FaceFacesMut, FaceHandle, FaceVertices, FaceVerticesMut, HalfEdge, HalfEdgeGraph,
    HalfEdgeHandle, Vertex, VertexEdges, VertexEdgesMut, VertexFaces, VertexFacesMut, VertexHandle,
    VertexInHalfEdges, VertexInHalfEdgesMut, VertexOutHalfEdges, VertexOutHalfEdgesMut,
    VertexVertex, VertexVertexMut,
};

macro_rules! fn_type {
    (
        $(#[$meta:meta])*
        struct $name:ident;

        $(#[$meta_mut:meta])*
        struct $name_mut:ident;

        handle: $handle:ty;
        type: $type:ident;
        map: $map:ident;

        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub struct $name<'graph, DataTypes: Data> {
            pub(crate) graph: &'graph HalfEdgeGraph<DataTypes>,
            pub(crate) handle: $handle,
        }

        impl<'graph, DataTypes: Data> $name<'graph, DataTypes>
        {
            pub(crate) fn new(graph: &'graph HalfEdgeGraph<DataTypes>, handle: $handle) -> Self {
                debug_assert!(graph.$map.contains_key(handle));

                Self {
                    graph,
                    handle
                }
            }

            pub fn handle(&self) -> $handle {
                self.handle
            }

            fn get(&self) -> &$type<DataTypes::$type> {
                // @todo investigate if get_unchecked is worth it
                //  (self.handle _should_ always be valid)
                &self.graph.$map.get(self.handle).unwrap()
            }

            fn_type!(__impl_props $name; $($rest)*);
        }
        fn_type!(__impl_common $name; handle: $handle; type: $type;);


        impl<'graph, DataTypes: Data> std::cmp::PartialEq<$name<'graph, DataTypes>> for $name<'graph, DataTypes> {
            fn eq(&self, other: &Self) -> bool {
                self.handle == other.handle
            }
        }


        $(#[$meta_mut])*
        pub struct $name_mut<'graph, DataTypes: Data> {
            pub(crate) graph: &'graph mut HalfEdgeGraph<DataTypes>,
            pub(crate) handle: $handle,
        }

        impl<'graph, DataTypes: Data> $name_mut<'graph, DataTypes>
        {
            pub(crate) fn new(graph: &'graph mut HalfEdgeGraph<DataTypes>, handle: $handle) -> Self {
                debug_assert!(graph.$map.contains_key(handle));

                Self {
                    graph,
                    handle
                }
            }

            pub fn handle(&self) -> $handle {
                self.handle
            }

            fn get(&self) -> &$type<DataTypes::$type> {
                // @todo investigate if get_mut is worth it
                //  (self.handle _should_ always be valid)
                self.graph.$map.get(self.handle).unwrap()
            }

            fn get_mut(&mut self) -> &mut $type<DataTypes::$type> {
                // @todo investigate if get_unchecked_mut is worth it
                //  (self.handle _should_ always be valid)
                self.graph.$map.get_mut(self.handle).unwrap()
            }

            fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
        }
        fn_type!(__impl_common_mut $name_mut; handle: $handle; type: $type;);

        impl<'graph, DataTypes: Data> From<$name_mut<'graph, DataTypes>> for $name<'graph, DataTypes> {
            fn from(item: $name_mut<'graph, DataTypes>) -> Self {
                Self::new(item.graph, item.handle)
            }
        }
    };

    (
        __impl_common $name:ident;
        handle: $handle:ty;
        type: $type:ident;
    ) => {
        impl<'graph, DataTypes: Data> std::fmt::Debug for $name<'graph, DataTypes> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.handle.0).finish()
            }
        }

        impl<'graph, DataTypes: Data> std::cmp::PartialEq<$handle> for $name<'graph, DataTypes> {
            fn eq(&self, other: &$handle) -> bool {
                self.handle == *other
            }
        }

        impl<'graph, DataTypes: Data> std::cmp::PartialEq<$name<'graph, DataTypes>> for $handle {
            fn eq(&self, other: &$name<'graph, DataTypes>) -> bool {
                *self == other.handle
            }
        }

        impl<'graph, DataTypes: Data> std::ops::Deref for $name<'graph, DataTypes> {
            type Target = DataTypes::$type;

            fn deref(&self) -> &Self::Target {
                &Self::get(self).data
            }
        }
    };

    (
        __impl_common_mut $name:ident;
        handle: $handle:ty;
        type: $type:ident;
    ) => {
        fn_type!(__impl_common $name; handle: $handle; type: $type;);

        impl<'graph, DataTypes: Data> std::ops::DerefMut for $name<'graph, DataTypes> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut Self::get_mut(self).data
            }
        }
    };

    /**************************
     * Properties             *
     **************************/

    ( // No more properties
        __impl_props $name:ident;
    ) => {};

    ( // Property
        __impl_props $name:ident;
        $(#[$meta:meta])*
        $prop:ident, $prop_mut:ident, $prop_as:ident -> $type:ident, $type_mut:ident;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $prop(&self) -> $type<'graph, DataTypes> {
            $type::new(self.graph, Self::get(self).$prop)
        }

        pub fn $prop_as(&self) -> $type<'graph, DataTypes> {
            $type::new(self.graph, Self::get(self).$prop)
        }

        fn_type!(__impl_props $name; $($rest)*);
    };

    ( // Optional property
        __impl_props $name:ident;
        $(#[$meta:meta])*
        $prop:ident, $prop_mut:ident, $prop_as:ident -> Option<$type:ident>, Option<$type_mut:ident>;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $prop(&self) -> Option<$type<'graph, DataTypes>> {
            let prop = Self::get(self).$prop;
            if prop.is_null() {
                None
            }
            else {
                Some($type::new(self.graph, prop))
            }
        }

        pub fn $prop_as(&self) -> Option<$type<'graph, DataTypes>> {
            let prop = Self::get(self).$prop;
            if prop.is_null() {
                None
            }
            else {
                Some($type::new(self.graph, prop))
            }

        }

        fn_type!(__impl_props $name; $($rest)*);
    };

    ( // Function
        __impl_props $name:ident;
        $(#[$meta:meta])*
        fn $fun:ident(&self) -> $ret:ident;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $fun(&self) -> $ret<'graph, DataTypes> {
            // FIXME
            $ret::new($name::new(self.graph, self.handle))
        }

        fn_type!(__impl_props $name; $($rest)*);
    };

    ( // Mutable function -- Ignore (not mutable type)
        __impl_props $name:ident;
        $(#[$meta:meta])*
        fn $prop:ident(&mut self) -> $ret:ty;
        $($rest:tt)*
    ) => {
        fn_type!(__impl_props $name; $($rest)*);
    };

    /**************************
     * Mutable properies      *
     **************************/
    ( // No more properties
        __impl_props_mut $name:ident:$name_mut:ident;
    ) => {};

    ( // Mutable property
        __impl_props_mut $name:ident:$name_mut:ident;
        $(#[$meta:meta])*
        $prop:ident, $prop_mut:ident, $prop_as:ident -> $type:ident, $type_mut:ident;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $prop(&mut self) -> $type<'_, DataTypes> {
            $type::new(self.graph, Self::get(self).$prop)
        }

        $(#[$meta])*
        pub fn $prop_mut(&mut self) -> $type_mut<'_, DataTypes> {
            $type_mut::new(self.graph, Self::get(self).$prop)
        }

        $(#[$meta])*
        pub fn $prop_as(self) -> $type_mut<'graph, DataTypes> {
            $type_mut::new(self.graph, Self::get(&self).$prop)
        }

        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };

    ( // Mutable optional property
        __impl_props_mut $name:ident:$name_mut:ident;
        $(#[$meta:meta])*
        $prop:ident, $prop_mut:ident, $prop_as:ident -> Option<$type:ident>, Option<$type_mut:ident>;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $prop(&self) -> Option<$type<'_, DataTypes>> {
            let prop = Self::get(self).$prop;
            if prop.is_null() {
                None
            }
            else {
                Some($type::new(self.graph, prop))
            }
        }

        $(#[$meta])*
        pub fn $prop_mut(&mut self) -> Option<$type_mut<'_, DataTypes>> {
            let prop = Self::get(self).$prop;
            if prop.is_null() {
                None
            }
            else {
                Some($type_mut::new(self.graph, prop))
            }
        }

        $(#[$meta])*
        pub fn $prop_as(self) -> Option<$type_mut<'graph, DataTypes>> {
            let prop = Self::get(&self).$prop;
            if prop.is_null() {
                None
            }
            else {
                Some($type_mut::new(self.graph, prop))
            }
        }

        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };

    ( // Function
        __impl_props_mut $name:ident:$name_mut:ident;
        $(#[$meta:meta])*
        fn $fun:ident(&self) -> $ret:ident;
        $($rest:tt)*
    ) => {
        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };

    ( // Mutable function
        __impl_props_mut $name:ident:$name_mut:ident;
        $(#[$meta:meta])*
        fn $fun:ident(&mut self) -> $ret:ident;
        $($rest:tt)*
    ) => {
        pub fn $fun(&mut self) -> $ret<DataTypes> {
            $ret::new($name_mut::new(self.graph, self.handle))
        }
        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };
}

fn_type!(
    /// Wrapper class for HalfEdgeHandle.
    /// It provides a simple interface for dealing with interfaces.
    /// See [*Fn](crate#function-set---shared)
    ///
    /// # Example
    ///```
    ///    use half_edge_graph::*;
    ///    /// create dummy graph
    ///    let mut graph = HalfEdgeGraph::<()>::default();
    ///    let v1 = graph.new_vertex(());
    ///    let v2 = graph.new_vertex(());
    ///    let edge = graph.new_edge(v1, v2, ());
    ///    // find a half edge between 2 vertices
    ///    let hedge_handle = graph.find_half_edge(v1, v2).unwrap();
    ///    let hedge = graph.half_edge(hedge_handle).unwrap();
    ///    // Walk the graph
    ///    assert_eq!(hedge.vertex(), v2);
    ///    assert_eq!(hedge.pair().vertex(), v1);
    ///```
    struct HalfEdgeFn;
    /// Mutable wrapper class for HalfEdgeHandle.
    /// See [HalfEdgeFn], [*FnMut](crate#function-set---mutable)
    struct HalfEdgeFnMut;

    handle: HalfEdgeHandle;
    type: HalfEdge;
    map: half_edges;

    /// Get the pair half edge
    pair, pair_mut, as_pair -> HalfEdgeFn, HalfEdgeFnMut;

    /// Get the next half edge
    #[allow(clippy::should_implement_trait)]
    next, next_mut, as_next -> HalfEdgeFn, HalfEdgeFnMut;

    /// Get the previus half edge
    prev, prev_mut, as_prev -> HalfEdgeFn, HalfEdgeFnMut;

    /// Get the vertex of the half edge
    vertex, vertex_mut, as_vertex -> VertexFn, VertexFnMut;

    /// Get the edge of the half edge
    edge, edge_mut, as_edge -> EdgeFn, EdgeFnMut;

    /// Get the face of the half edge
    face, face_mut, as_face -> Option<FaceFn>, Option<FaceFnMut>;
);

fn_type!(
    struct VertexFn;
    struct VertexFnMut;

    handle: VertexHandle;
    type: Vertex;
    map: vertices;

    hedge, hedge_mut, as_hedge -> Option<HalfEdgeFn>, Option<HalfEdgeFnMut>;

    fn in_half_edges(&self) -> VertexInHalfEdges;
    fn in_half_edges_mut(&mut self) -> VertexInHalfEdgesMut;

    fn out_half_edges(&self) -> VertexOutHalfEdges;
    fn out_half_edges_mut(&mut self) -> VertexOutHalfEdgesMut;

    fn vertices(&self) -> VertexVertex;
    fn vertices_mut(&mut self) -> VertexVertexMut;

    fn edges(&self) -> VertexEdges;
    fn edges_mut(&mut self) -> VertexEdgesMut;

    fn faces(&self) -> VertexFaces;
    fn faces_mut(&mut self) -> VertexFacesMut;
);

fn_type!(
    struct EdgeFn;
    struct EdgeFnMut;

    handle: EdgeHandle;
    type: Edge;
    map: edges;

    hedge, hedge_mut, as_hedge -> HalfEdgeFn, HalfEdgeFnMut;

    fn faces(&self) -> EdgeFaces;
    fn faces_mut(&mut self) -> EdgeFacesMut;
);

impl<'graph, DataTypes: Data> EdgeFn<'graph, DataTypes> {
    pub fn vertices(&self) -> [VertexFn<'graph, DataTypes>; 2] {
        let edge = EdgeFn::new(self.graph, self.handle);

        let hedge = edge.hedge();
        let pair = hedge.pair();

        [hedge.vertex(), pair.vertex()]
    }

    // @todo figure out how vertices_mut can be implemented soundly
    // The problem is that the straigt forward way gives out multiple
    // mutable references to `graph`
    // We would need a way to limit access to one vertex at the time
    // pub fn vertices_mut(&self) -> [VertexFnMut<'graph, DataTypes>; 2] {
    // }
}

fn_type!(
    struct FaceFn;
    struct FaceFnMut;

    handle: FaceHandle;
    type: Face;
    map: faces;

    hedge, hedge_mut, as_hedge -> HalfEdgeFn, HalfEdgeFnMut;

    fn vertices(&self) -> FaceVertices;
    fn vertices_mut(&mut self) -> FaceVerticesMut;

    fn edges(&self) -> FaceEdges;
    fn edges_mut(&mut self) -> FaceEdgesMut;

    fn faces(&self) -> FaceFaces;
    fn faces_mut(&mut self) -> FaceFacesMut;
);
