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

        $(#[$meta_data:meta])*
        struct $name_data:ident;

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

            fn_type!(__impl_props $name 'graph; $($rest)*);
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

            fn_type!(__impl_props $name '_; $($rest)*);
            fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
        }
        fn_type!(__impl_common_mut $name_mut; handle: $handle; type: $type;);

        impl<'graph, DataTypes: Data> From<$name_mut<'graph, DataTypes>> for $name<'graph, DataTypes> {
            fn from(item: $name_mut<'graph, DataTypes>) -> Self {
                Self::new(item.graph, item.handle)
            }
        }

        $(#[$meta_data])*
        pub struct $name_data<'graph, DataTypes: Data> {
            pub(crate) data: &'graph mut $type<DataTypes::$type>,
            pub(crate) handle: $handle,
        }

        impl<'graph, DataTypes: Data> $name_data<'graph, DataTypes>
        {
            pub(crate) fn new(data: &'graph mut $type<DataTypes::$type>, handle: $handle) -> Self {
                Self {
                    data,
                    handle
                }
            }

            pub fn handle(&self) -> $handle {
                self.handle
            }

            fn get(&self) -> &$type<DataTypes::$type> {
                self.data
            }

            fn get_mut(&mut self) -> &mut $type<DataTypes::$type> {
                self.data
            }
        }

        fn_type!(__impl_common_mut $name_data; handle: $handle; type: $type;);
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
        __impl_props $name:ident $lifetime:lifetime;
    ) => {};

    ( // Property
        __impl_props $name:ident $lifetime:lifetime;
        $prop:ident(&self) -> $type:ident;
        $($rest:tt)*
    ) => {
        pub fn $prop(&self) -> $type<$lifetime, DataTypes> {
            $type::new(self.graph, Self::get(self).$prop)
        }

        fn_type!(__impl_props $name $lifetime; $($rest)*);
    };

    ( // Optional property
        __impl_props $name:ident $lifetime:lifetime;
        $prop:ident(&self) -> Option<$type:ident>;
        $($rest:tt)*
    ) => {
        pub fn $prop(&self) -> Option<$type<$lifetime, DataTypes>> {
            let prop = Self::get(self).$prop;
            if prop.is_null() {
                None
            }
            else {
                Some($type::new(self.graph, prop))
            }

        }

        fn_type!(__impl_props $name $lifetime; $($rest)*);
    };
    ( // Function
        __impl_props $name:ident $lifetime:lifetime;
        fn $fun:ident(&self) -> $ret:ident;
        $($rest:tt)*
    ) => {
        pub fn $fun(&self) -> $ret<$lifetime, DataTypes> {
            // FIXME
            $ret::new($name::new(self.graph, self.handle))
        }

        fn_type!(__impl_props $name $lifetime; $($rest)*);
    };

    ( // Mutable property -- Ignore (not mutable type)
        __impl_props $name:ident $lifetime:lifetime;
        $fun:ident:$fun_as:ident:$prop:ident(&mut self) -> $type:ident;
        $($rest:tt)*
    ) => {
        fn_type!(__impl_props $name $lifetime; $($rest)*);
    };
    ( // Mutable optional property -- Ignore (not mutable type)
        __impl_props $name:ident $lifetime:lifetime;
        $fun:ident:$fun_as:ident:$prop:ident(&mut self) -> Option<$type:ident>;
        $($rest:tt)*
    ) => {
        fn_type!(__impl_props $name $lifetime; $($rest)*);
    };
    ( // Mutable function -- Ignore (not mutable type)
        __impl_props $name:ident $lifetime:lifetime;
        fn $prop:ident(&mut self) -> $ret:ty;
        $($rest:tt)*
    ) => {
        fn_type!(__impl_props $name $lifetime; $($rest)*);
    };

    /**************************
     * Mutable properies      *
     **************************/
    ( // No more properties
        __impl_props_mut $name:ident:$name_mut:ident;
    ) => {};

    ( // Property
        __impl_props_mut $name:ident:$name_mut:ident;
        $prop:ident(&self) -> $type:ident;
        $($rest:tt)*
    ) => {
        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };

    ( // Optional property
        __impl_props_mut $name:ident:$name_mut:ident;
        $prop:ident(&self) -> Option<$type:ident>;
        $($rest:tt)*
    ) => {
        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };

    ( // Mutable property
        __impl_props_mut $name:ident:$name_mut:ident;
        $fun:ident:$fun_as:ident:$prop:ident(&mut self) -> $type:ident;
        $($rest:tt)*
    ) => {
        pub fn $fun(&mut self) -> $type<'_, DataTypes> {
            $type::new(self.graph, Self::get(self).$prop)
        }

        pub fn $fun_as(self) -> $type<'graph, DataTypes> {
            $type::new(self.graph, Self::get(&self).$prop)
        }

        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };

    ( // Mutable optional property
        __impl_props_mut $name:ident:$name_mut:ident;
        $fun:ident:$fun_as:ident:$prop:ident(&mut self) -> Option<$type:ident>;
        $($rest:tt)*
    ) => {
        pub fn $fun(&mut self) -> Option<$type<'_, DataTypes>> {
            let prop = Self::get(self).$prop;
            if prop.is_null() {
                None
            }
            else {
                Some($type::new(self.graph, prop))
            }
        }

        pub fn $fun_as(self) -> Option<$type<'graph, DataTypes>> {
            let prop = Self::get(&self).$prop;
            if prop.is_null() {
                None
            }
            else {
                Some($type::new(self.graph, prop))
            }
        }

        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };

    ( // Function
        __impl_props_mut $name:ident:$name_mut:ident;
        fn $fun:ident(&self) -> $ret:ident;
        $($rest:tt)*
    ) => {
        fn_type!(__impl_props_mut $name:$name_mut; $($rest)*);
    };

    ( // Mutable function
        __impl_props_mut $name:ident:$name_mut:ident;
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
    struct HalfEdgeFn;
    struct HalfEdgeFnMut;
    struct HalfEdgeFnData;

    handle: HalfEdgeHandle;
    type: HalfEdge;
    map: half_edges;

    pair(&self) -> HalfEdgeFn;
    pair_mut:as_pair:pair(&mut self) -> HalfEdgeFnMut;

    next(&self) -> HalfEdgeFn;
    next_mut:as_next:next(&mut self) -> HalfEdgeFnMut;

    prev(&self) -> HalfEdgeFn;
    prev_mut:as_prev:prev(&mut self) -> HalfEdgeFnMut;

    vertex(&self) -> VertexFn;
    vertex_mut:as_vertex:vertex(&mut self) -> VertexFnMut;

    edge(&self) -> EdgeFn;
    edge_mut:as_edge:edge(&mut self) -> EdgeFnMut;

    face(&self) -> Option<FaceFn>;
    face_mut:as_face:face(&mut self) -> Option<FaceFnMut>;
);

fn_type!(
    struct VertexFn;
    struct VertexFnMut;
    struct VertexFnData;

    handle: VertexHandle;
    type: Vertex;
    map: vertices;

    hedge(&self) -> Option<HalfEdgeFn>;
    hedge_mut:as_hedge:hedge(&mut self) -> Option<HalfEdgeFnMut>;

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
    struct EdgeFnData;

    handle: EdgeHandle;
    type: Edge;
    map: edges;

    hedge(&self) -> HalfEdgeFn;
    hedge_mut:as_hedge:hedge(&mut self) -> HalfEdgeFnMut;

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
    struct FaceFnData;

    handle: FaceHandle;
    type: Face;
    map: faces;

    hedge(&self) -> HalfEdgeFn;
    hedge_mut:as_hedge:hedge(&mut self) -> HalfEdgeFnMut;

    fn vertices(&self) -> FaceVertices;
    fn vertices_mut(&mut self) -> FaceVerticesMut;

    fn edges(&self) -> FaceEdges;
    fn edges_mut(&mut self) -> FaceEdgesMut;

    fn faces(&self) -> FaceFaces;
    fn faces_mut(&mut self) -> FaceFacesMut;
);
