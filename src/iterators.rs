use crate::{
    Data, EdgeFn, EdgeFnMut, EdgeHandle, FaceFn, FaceFnMut, FaceHandle, HalfEdgeFn, HalfEdgeFnMut,
    HalfEdgeGraph, HalfEdgeHandle, VertexFn, VertexFnMut, VertexHandle,
};

pub struct VerticesMut<'graph, DataTypes: Data> {
    graph: &'graph mut HalfEdgeGraph<DataTypes>,
    handles: Vec<VertexHandle>,
    current: usize,
}

impl<'graph, DataTypes: Data> VerticesMut<'graph, DataTypes> {
    pub fn new(graph: &'graph mut HalfEdgeGraph<DataTypes>) -> Self {
        let handles = graph.vertices.keys().collect();
        Self {
            graph,
            handles,
            current: 0,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<VertexFnMut<'_, DataTypes>> {
        self.current += 1;
        if let Some(handle) = self.handles.get(self.current - 1).cloned() {
            Some(VertexFnMut::new(self.graph, handle))
        } else {
            None
        }
    }
}

pub struct HalfEdgesMut<'graph, DataTypes: Data> {
    graph: &'graph mut HalfEdgeGraph<DataTypes>,
    handles: Vec<HalfEdgeHandle>,
    current: usize,
}

impl<'graph, DataTypes: Data> HalfEdgesMut<'graph, DataTypes> {
    pub fn new(graph: &'graph mut HalfEdgeGraph<DataTypes>) -> Self {
        let handles = graph.half_edges.keys().collect();
        Self {
            graph,
            handles,
            current: 0,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<HalfEdgeFnMut<'_, DataTypes>> {
        self.current += 1;
        if let Some(handle) = self.handles.get(self.current - 1).cloned() {
            Some(HalfEdgeFnMut::new(self.graph, handle))
        } else {
            None
        }
    }
}
pub struct EdgesMut<'graph, DataTypes: Data> {
    graph: &'graph mut HalfEdgeGraph<DataTypes>,
    handles: Vec<EdgeHandle>,
    current: usize,
}

impl<'graph, DataTypes: Data> EdgesMut<'graph, DataTypes> {
    pub fn new(graph: &'graph mut HalfEdgeGraph<DataTypes>) -> Self {
        let handles = graph.edges.keys().collect();
        Self {
            graph,
            handles,
            current: 0,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<EdgeFnMut<'_, DataTypes>> {
        self.current += 1;
        if let Some(handle) = self.handles.get(self.current - 1).cloned() {
            Some(EdgeFnMut::new(self.graph, handle))
        } else {
            None
        }
    }
}

pub struct FacesMut<'graph, DataTypes: Data> {
    graph: &'graph mut HalfEdgeGraph<DataTypes>,
    handles: Vec<FaceHandle>,
    current: usize,
}

impl<'graph, DataTypes: Data> FacesMut<'graph, DataTypes> {
    pub fn new(graph: &'graph mut HalfEdgeGraph<DataTypes>) -> Self {
        let handles = graph.faces.keys().collect();
        Self {
            graph,
            handles,
            current: 0,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<FaceFnMut<'_, DataTypes>> {
        self.current += 1;
        if let Some(handle) = self.handles.get(self.current - 1).cloned() {
            Some(FaceFnMut::new(self.graph, handle))
        } else {
            None
        }
    }
}

macro_rules! create_iterator {
    (
        struct $name:ident;
        struct $name_mut:ident;

        init: $init:ident;
        init_mut: $init_mut:ident;

        out: $out:ident;
        out_mut: $out_mut:ident;

        fn init($init_handle:ident) {
            $($init_fn:tt)+
        }

        fn get($get_handle:ident) {
            $($get_fn:tt)+
        }

        fn next($next_handle:ident) {
            $($next_fn:tt)+
        }

        fn valid($valid_handle:ident) {
            $($valid_fn:tt)+
        }
    ) => {
        pub struct $name<'graph, DataTypes: Data> {
            graph: &'graph HalfEdgeGraph<DataTypes>,
            head: HalfEdgeHandle,
            current: Option<HalfEdgeHandle>,


            #[cfg(debug_assertions)]
            // Used to detect corruption, durring the iteration
            // if we ever visit the same handle twice, something have gone wrong
            visited: Vec<HalfEdgeHandle>,
        }


        impl<'graph, DataTypes: Data> $name<'graph, DataTypes> {
            pub fn new(handle: $init<'graph, DataTypes>) -> Self {
                let graph = handle.graph;
                let init = Self::_init(handle).map(|i| i.handle());
                let mut iter = Self {
                    graph,
                    head: init.unwrap_or_else(HalfEdgeHandle::null),
                    current: init,

                    #[cfg(debug_assertions)]
                    visited: vec![],
                };

                if let Some(current) = iter.current {
                    // Make sure we start on a valid element
                    if !Self::_valid(HalfEdgeFn::new(iter.graph, current)) {
                        iter._advance_next();
                    }
                }

                iter
            }

            create_iterator!(
                __impl_common;
                init: $init;
                init_mut: $init_mut;

                out: $out;
                out_mut: $out_mut;

                fn init($init_handle) {
                    $($init_fn)+
                }

                fn next($next_handle) {
                    $($next_fn)+
                }

                fn valid($valid_handle) {
                    $($valid_fn)+
                }
            );
        }

        impl<'graph, DataTypes: Data> Iterator for $name<'graph, DataTypes> {
            type Item = $out<'graph, DataTypes>;

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(current) = self.current {
                    self._advance_next();

                    let $get_handle = HalfEdgeFn::new(self.graph, current);
                    Some({$($get_fn)+})
                }
                else {
                    None
                }
            }
        }

        pub struct $name_mut<'graph, DataTypes: Data> {
            graph: &'graph mut HalfEdgeGraph<DataTypes>,
            head: HalfEdgeHandle,
            current: Option<HalfEdgeHandle>,


            #[cfg(debug_assertions)]
            // Used to detect corruption, durring the iteration
            // if we ever visit the same handle twice, something have gone wrong
            visited: Vec<HalfEdgeHandle>,
        }

        impl<'graph, DataTypes: Data> $name_mut<'graph, DataTypes> {
            pub fn new(handle: $init_mut<'graph, DataTypes>) -> Self {
                let init = Self::_init($init::new(handle.graph, handle.handle)).map(|i| i.handle());

                let mut iter = Self {
                    graph: handle.graph,
                    head: init.unwrap_or_else(HalfEdgeHandle::null),
                    current: init,

                    #[cfg(debug_assertions)]
                    visited: vec![],
                };

                if let Some(current) = iter.current {
                    // Make sure we start on a valid element
                    if !Self::_valid(HalfEdgeFn::new(iter.graph, current)) {
                        iter._advance_next();
                    }
                }

                iter
            }
            // ignore warning about implementing next as `Iterator` - its not possible due to lifetime constraint
            #[allow(renamed_and_removed_lints)]
            #[allow(should_implement_trait)]
            pub fn next(&mut self) -> Option<$out_mut<'_, DataTypes>> {
                if let Some(current) = self.current {
                    self._advance_next();

                    #[allow(unused_mut)]
                    let mut $get_handle = HalfEdgeFnMut::new(self.graph, current);

                    Some($($get_fn)+)
                }
                else {
                    None
                }
            }

            create_iterator!(
                __impl_common;
                init: $init;
                init_mut: $init_mut;

                out: $out;
                out_mut: $out_mut;

                fn init($init_handle) {
                    $($init_fn)+
                }

                fn next($next_handle) {
                    $($next_fn)+
                }

                fn valid($valid_handle) {
                    $($valid_fn)+
                }
            );
        }
    };

    (
        __impl_common;
        init: $init:ident;
        init_mut: $init_mut:ident;

        out: $out:ident;
        out_mut: $out_mut:ident;

        fn init($init_handle:ident) {
            $($init_fn:tt)+
        }

        fn next($next_handle:ident) {
            $($next_fn:tt)+
        }

        fn valid($valid_handle:ident) {
            $($valid_fn:tt)+
        }
    ) => {
        fn _advance_next(&mut self) {
            while let Some(current) = self.current {
                let next = Self::_next(HalfEdgeFn::new(self.graph, current));

                #[cfg(debug_assertions)] {
                    assert!(!self.visited.contains(&next.handle()));
                    self.visited.push(next.handle());
                }

                if next == self.head {
                    self.current = None;
                }
                else {
                    self.current = Some(next.handle());

                    if Self::_valid(next) {
                        break
                    }
                }

            }
        }

        fn _init($init_handle: $init<'_, DataTypes>) -> Option<HalfEdgeFn<'_, DataTypes>> {
            $($init_fn)+
        }

        fn _init_mut(handle: $init_mut<'_, DataTypes>) -> Option<HalfEdgeFn<'_, DataTypes>> {
            let $init_handle = $init::new(handle.graph, handle.handle);
            $($init_fn)+
        }

        fn _next($next_handle: HalfEdgeFn<'_, DataTypes>) -> HalfEdgeFn<'_, DataTypes> {
            $($next_fn)+
        }

        fn _valid($valid_handle: HalfEdgeFn<'_, DataTypes>) -> bool {
            $($valid_fn)+
        }
    };
}

create_iterator!(
    struct VertexOutHalfEdges;
    struct VertexOutHalfEdgesMut;

    init: VertexFn;
    init_mut: VertexFnMut;

    out: HalfEdgeFn;
    out_mut: HalfEdgeFnMut;

    fn init(vertex) {
        vertex.hedge()
    }

    fn get(current) {
        current
    }

    fn next(current) {
        current.pair().next()
    }

    fn valid(_current) {
        true
    }
);

create_iterator!(
    struct VertexInHalfEdges;
    struct VertexInHalfEdgesMut;

    init: VertexFn;
    init_mut: VertexFnMut;

    out: HalfEdgeFn;
    out_mut: HalfEdgeFnMut;

    fn init(vertex) {
        Some(vertex.hedge()?.pair())
    }

    fn get(current) {
        current
    }

    fn next(current)  {
        current.next().pair()
    }

    fn valid(_current) {
        true
    }
);

create_iterator!(
    struct VertexVertex;
    struct VertexVertexMut;

    init: VertexFn;
    init_mut: VertexFnMut;

    out: VertexFn;
    out_mut: VertexFnMut;

    fn init(vertex) {
        vertex.hedge()
    }

    fn get(current) {
        current.as_vertex()
    }

    fn next(current)  {
        current.pair().next()
    }

    fn valid(_current) {
        true
    }
);

create_iterator!(
    struct VertexEdges;
    struct VertexEdgesMut;

    init: VertexFn;
    init_mut: VertexFnMut;

    out: EdgeFn;
    out_mut: EdgeFnMut;

    fn init(vertex) {
        vertex.hedge()
    }

    fn get(current) {
        current.as_edge()
    }

    fn next(current)  {
        current.pair().next()
    }

    fn valid(_current) {
        true
    }
);

create_iterator!(
    struct VertexFaces;
    struct VertexFacesMut;

    init: VertexFn;
    init_mut: VertexFnMut;

    out: FaceFn;
    out_mut: FaceFnMut;

    fn init(vertex) {
        vertex.hedge()
    }

    fn get(current) {
        current.as_face().unwrap()
    }

    fn next(current)  {
        current.as_pair().next()
    }

    fn valid(current) {
        current.as_face().is_some()
    }
);

create_iterator!(
    struct EdgeFaces;
    struct EdgeFacesMut;

    init: EdgeFn;
    init_mut: EdgeFnMut;

    out: FaceFn;
    out_mut: FaceFnMut;

    fn init(edge) {
        Some(edge.hedge())
    }

    fn get(current) {
        current.as_face().unwrap()
    }

    fn next(current)  {
        current.pair()
    }

    fn valid(current) {
        current.face().is_some()
    }
);

create_iterator!(
    struct FaceVertices;
    struct FaceVerticesMut;

    init: FaceFn;
    init_mut: FaceFnMut;

    out: VertexFn;
    out_mut: VertexFnMut;

    fn init(face) {
        Some(face.hedge())
    }

    fn get(current) {
        current.as_vertex()
    }

    fn next(current)  {
        current.next()
    }

    fn valid(_current) {
        true
    }
);

create_iterator!(
    struct FaceEdges;
    struct FaceEdgesMut;

    init: FaceFn;
    init_mut: FaceFnMut;

    out: EdgeFn;
    out_mut: EdgeFnMut;

    fn init(face) {
        Some(face.hedge())
    }

    fn get(current) {
        current.as_edge()
    }

    fn next(current)  {
        current.next()
    }

    fn valid(_current) {
        true
    }
);

create_iterator!(
    struct FaceFaces;
    struct FaceFacesMut;

    init: FaceFn;
    init_mut: FaceFnMut;

    out: FaceFn;
    out_mut: FaceFnMut;

    fn init(face) {
        Some(face.hedge())
    }

    fn get(current) {
        current.as_pair().as_face().unwrap()
    }

    fn next(current)  {
        current.next()
    }

    fn valid(current) {
        current.pair().face().is_some()
    }
);
