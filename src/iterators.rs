use super::{
    Data, EdgeFn, EdgeHandle, FaceFn, FaceHandle, HalfEdgeFn, HalfEdgeGraph, HalfEdgeHandle,
    VertexFn, VertexHandle,
};

impl<'mesh, DataTypes: Data> HalfEdgeGraph<DataTypes> {
    pub fn iter_vertices<'self_, 'iter>(
        &'self_ self,
    ) -> impl Iterator<Item = VertexFn<'iter, DataTypes>>
    where
        'self_: 'iter,
    {
        self.vertices
            .keys()
            .map(move |handle| VertexFn::<'iter, DataTypes>::new(self, handle))
    }

    pub fn iter_edges<'self_, 'iter>(&'self_ self) -> impl Iterator<Item = EdgeFn<'iter, DataTypes>>
    where
        'self_: 'iter,
    {
        self.edges
            .keys()
            .map(move |handle| EdgeFn::<'iter, DataTypes>::new(self, handle))
    }

    pub fn iter_faces<'self_, 'iter>(&'self_ self) -> impl Iterator<Item = FaceFn<'iter, DataTypes>>
    where
        'self_: 'iter,
    {
        self.faces
            .keys()
            .map(move |handle| FaceFn::<'iter, DataTypes>::new(self, handle))
    }

    pub fn iter_half_edges<'self_, 'iter>(
        &'self_ self,
    ) -> impl Iterator<Item = HalfEdgeFn<'iter, DataTypes>>
    where
        'self_: 'iter,
    {
        self.half_edges
            .keys()
            .map(move |handle| HalfEdgeFn::<'iter, DataTypes>::new(self, handle))
    }

    pub fn iter_vertices_data_mut(
        &mut self,
    ) -> impl Iterator<Item = (VertexHandle, &mut DataTypes::Vertex)> {
        self.vertices.iter_mut().map(|(h, d)| (h, &mut d.data))
    }

    pub fn iter_edges_data_mut(
        &mut self,
    ) -> impl Iterator<Item = (EdgeHandle, &mut DataTypes::Edge)> {
        self.edges.iter_mut().map(|(h, d)| (h, &mut d.data))
    }

    pub fn iter_half_edges_data_mut(
        &mut self,
    ) -> impl Iterator<Item = (HalfEdgeHandle, &mut DataTypes::HalfEdge)> {
        self.half_edges.iter_mut().map(|(h, d)| (h, &mut d.data))
    }

    pub fn iter_faces_data_mut(
        &mut self,
    ) -> impl Iterator<Item = (FaceHandle, &mut DataTypes::Face)> {
        self.faces.iter_mut().map(|(h, d)| (h, &mut d.data))
    }
}

macro_rules! create_iterator {
    (
        $name:ident($init_ty:ident -> $res_ty:ident) {
            fn init($handle_init:ident: $_0:ident) -> Option<HalfEdgeFn> {
                $($init:tt)*
            }
            fn next($current_next:ident : HalfEdgeFn) -> HalfEdgeFn {
                $($next:tt)*
            }
            fn get($current_get:ident : HalfEdgeFn) -> $_1:ident {
                $($get:tt)*
            }

            $(
                fn valid($current_valid:ident: HalfEdgeFn) -> bool {
                    $($valid:tt)*
                }
            )?
        }
    ) => {
        pub struct $name<'mesh, DataTypes: Data> {
            mesh: &'mesh HalfEdgeGraph<DataTypes>,
            head: HalfEdgeHandle,
            current: HalfEdgeHandle,

            #[cfg(debug_assertions)]
            // Used to detect corruption, durring the iteration
            // if we every visit the same handle twice, something have gone wrong
            visited: Vec<HalfEdgeHandle>,

        }
        #[allow(unused)]
        impl<'mesh, DataTypes: Data> $name<'mesh, DataTypes> {
            pub fn new($handle_init: $init_ty<'mesh, DataTypes>) -> Self {
                let mesh = $handle_init.mesh();
                let head = Some($handle_init)
                    .and_then(|$handle_init| {$($init)*})
                    .map(|h| h.handle())
                    .unwrap_or_else(HalfEdgeHandle::null);

                let mut iter = Self {
                    mesh,
                    head,
                    current: head,
                    #[cfg(debug_assertions)]
                    visited: Vec::new(),
                };

                $( // make sure current is valid
                    while (!iter.current.is_null() && !{
                        let $current_valid = iter.mesh.half_edge(iter.current).unwrap();
                        $($valid)*
                    }) {
                        iter.advance_next();
                    }
                )?
                iter
            }

            fn advance_next(&mut self) {
                let next = {
                    let $current_next = self.mesh.half_edge(self.current).unwrap();
                    $($next)*
                }.handle();

                debug_assert!(!next.is_null());
                #[cfg(debug_assertions)] {
                    debug_assert!(!self.visited.contains(&next));
                    self.visited.push(next);
                }
                self.current = if next != self.head { next }  else { HalfEdgeHandle::null() };
            }
        }

        impl<'mesh, DataTypes: Data> std::iter::Iterator for $name<'mesh, DataTypes> {
            type Item = $res_ty<'mesh, DataTypes>;
            #[allow(unused)]
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    if self.current.is_null() {
                        return None;
                    }

                    let $current_get = self.mesh.half_edge(self.current).unwrap();

                    self.advance_next();
                    $( // make sure current is valid
                        while (!self.current.is_null() && !{
                            let $current_valid = self.mesh.half_edge(self.current).unwrap();
                            $($valid)*
                        }) {
                            self.advance_next();
                        }
                    )?

                    let valid = true;
                    if valid {
                        return Some({$($get)*})
                    }
                }
            }
        }

    };
}

create_iterator!(
    VertexOutHalfEdges (VertexFn -> HalfEdgeFn) {
        fn init(vertex: VertexFn) -> Option<HalfEdgeFn> {
            vertex.hedge()
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.pair().next()
        }
        fn get(current: HalfEdgeFn) -> HalfEdgeFn {
            current
        }
    }
);

create_iterator!(
    VertexInHalfEdges (VertexFn -> HalfEdgeFn) {
        fn init(vertex: VertexFn) -> Option<HalfEdgeFn> {
            Some(vertex.hedge()?.pair())
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.next().pair()
        }
        fn get(current: HalfEdgeFn) -> HalfEdgeFn {
            current
        }
    }
);

create_iterator!(
    VertexVertex (VertexFn -> VertexFn) {
        fn init(vertex: VertexFn) -> Option<HalfEdgeFn> {
            vertex.hedge()
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.pair().next()
        }
        fn get(current: HalfEdgeFn) -> EdgeFn {
            current.vertex()
        }
    }
);

create_iterator!(
    VertexEdges (VertexFn -> EdgeFn) {
        fn init(vertex: VertexFn) -> Option<HalfEdgeFn> {
            vertex.hedge()
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.pair().next()
        }
        fn get(current: HalfEdgeFn) -> EdgeFn {
            current.edge()
        }
    }
);

create_iterator!(
    VertexFaces (VertexFn -> FaceFn) {
        fn init(vertex: VertexFn) -> Option<HalfEdgeFn> {
            vertex.hedge()
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.pair().next()
        }
        fn get(current: HalfEdgeFn) -> FaceFn {
            current.face().unwrap()
        }
        fn valid(current: HalfEdgeFn) -> bool {
            current.face().is_some()
        }
    }
);

create_iterator!(
    EdgeFaces (EdgeFn -> FaceFn) {
        fn init(edge: EdgeFn) -> Option<HalfEdgeFn> {
            Some(edge.hedge())
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.pair()
        }
        fn get(current: HalfEdgeFn) -> FaceFn {
            current.face().unwrap()
        }
        fn valid(current: HalfEdgeFn) -> bool {
            current.face().is_some()
        }
    }
);

create_iterator!(
    FaceVertices (FaceFn -> VertexFn) {
        fn init(face: FaceFn) -> Option<HalfEdgeFn> {
            Some(face.hedge())
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.next()
        }
        fn get(current: HalfEdgeFn) -> FaceFn {
            current.vertex()
        }
    }
);

create_iterator!(
    FaceEdges (FaceFn -> EdgeFn) {
        fn init(face: FaceFn) -> Option<HalfEdgeFn> {
            Some(face.hedge())
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.next()
        }
        fn get(current: HalfEdgeFn) -> EdgeFn {
            current.edge()
        }
    }
);

create_iterator!(
    FaceFaces (FaceFn -> FaceFn) {
        fn init(face: FaceFn) -> Option<HalfEdgeFn> {
            Some(face.hedge())
        }
        fn next(current: HalfEdgeFn) -> HalfEdgeFn {
            current.next()
        }
        fn get(current: HalfEdgeFn) -> FaceFn {
            current.pair().face().unwrap()
        }
        fn valid(current: HalfEdgeFn) -> bool {
            current.pair().face().is_some()
        }
    }
);
