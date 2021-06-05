mod function_set;
mod iterators;
mod accessors;

use std::fmt::Debug;

use itertools::Itertools;
use slotmap::Key;

pub use function_set::{
    VertexFn, EdgeFn, HalfEdgeFn, FaceFn
};

// #[cfg(test)]
mod test;

slotmap::new_key_type!(
    pub struct FaceHandle;
    pub struct EdgeHandle;
    pub struct HalfEdgeHandle;
    pub struct VertexHandle;
);

impl FaceHandle {
    pub fn is_null(&self) -> bool {
        <Self as Key>::is_null(&self)
    }
    pub fn null() -> Self {
        <Self as Key>::null()
    }
}
impl EdgeHandle {
    pub fn is_null(&self) -> bool {
        <Self as Key>::is_null(&self)
    }
    pub fn null() -> Self {
        <Self as Key>::null()
    }
}
impl HalfEdgeHandle {
    pub fn is_null(&self) -> bool {
        <Self as Key>::is_null(&self)
    }
    pub fn null() -> Self {
        <Self as Key>::null()
    }
}
impl VertexHandle {
    pub fn is_null(&self) -> bool {
        <Self as Key>::is_null(&self)
    }
    pub fn null() -> Self {
        <Self as Key>::null()
    }
}




pub trait Data {
    type Face: Default;
    type Edge: Default;
    type HalfEdge: Default;
    type Vertex: Default;
}

impl Data for () {
    type Face = ();
    type Edge = ();
    type HalfEdge = ();
    type Vertex = ();
}

#[derive(Default, Debug)]
struct HalfEdge<Data> {
    data: Data,

    pair: HalfEdgeHandle,
    next: HalfEdgeHandle,
    prev: HalfEdgeHandle,

    vertex: VertexHandle,
    edge: EdgeHandle,
    face: FaceHandle,
}
#[derive(Default, Debug)]
struct Face<Data> {
    data: Data,
    hedge: HalfEdgeHandle,
}

#[derive(Default, Debug)]
struct Edge<Data> {
    data: Data,
    hedge: HalfEdgeHandle,
}

#[derive(Default, Debug)]
struct Vertex<Data> {
    data: Data,
    hedge: HalfEdgeHandle,
}

#[derive(Default, Debug)]
pub struct HalfEdgeGraph<DataTypes: Data> {
    half_edges: slotmap::SlotMap<HalfEdgeHandle, HalfEdge<DataTypes::HalfEdge>>,
    faces: slotmap::SlotMap<FaceHandle, Face<DataTypes::Face>>,
    edges: slotmap::SlotMap<EdgeHandle, Edge<DataTypes::Edge>>,
    vertices: slotmap::SlotMap<VertexHandle, Vertex<DataTypes::Vertex>>,
    data: DataTypes,
}

impl<DataTypes: Data> HalfEdgeGraph<DataTypes> {
    pub fn new_vertex(&mut self, data: DataTypes::Vertex) -> VertexHandle {
        let vertex = Vertex {
            data,
            ..Default::default()
        };
        self.vertices.insert(vertex)
    }

    pub fn new_edge(
        &mut self,
        v1: VertexHandle,
        v2: VertexHandle,
        data: DataTypes::Edge,
    ) -> Option<EdgeHandle> {
        if v1 == v2 {
            return None;
        }
        if self.find_edge(v1, v2).is_some() {
            // Only 1 edge is allowed between vertexes
            return None;
        }

        let vertex_1 = self.vertex(v1)?;
        let vertex_2 = self.vertex(v2)?;

        let v1_insertion = vertex_1
            .hedge()
            .map(|h| self.find_free_half_edge(h.handle()).ok_or(()))
            .transpose().ok()?;

        let v2_insertion = vertex_2
            .hedge()
            .map(|h| self.find_free_half_edge(h.handle()).ok_or(()))
            .transpose().ok()?;

        let e = self.edges.insert(Default::default());
        let h1 = self.half_edges.insert(Default::default());
        let h2 = self.half_edges.insert(Default::default());

        self.edges[e] = Edge { data, hedge: h1 };

        self.half_edges[h1] = HalfEdge {
            data: Default::default(),
            pair: h2,
            next: h2,
            prev: h2,

            vertex: v2,
            edge: e,
            face: FaceHandle::null(),
        };
        self.half_edges[h2] = HalfEdge {
            data: Default::default(),
            pair: h1,
            next: h1,
            prev: h1,

            vertex: v1,
            edge: e,
            face: FaceHandle::null(),
        };

        if let Some(v1_insertion) = v1_insertion {
            debug_assert_eq!(self.half_edge(v1_insertion).unwrap().pair().vertex(), v1);

            let e1 = v1_insertion;
            let e2 = self.half_edges[e1].prev;

            self.half_edges[e2].next = h1;
            self.half_edges[h1].prev = e2;

            self.half_edges[h2].next = e1;
            self.half_edges[e1].prev = h2;
        } else {
            debug_assert!(self.vertices[v1].hedge.is_null());
            self.vertices[v1].hedge = h1;
        }

        if let Some(v2_insertion) = v2_insertion {
            debug_assert_eq!(self.half_edge(v2_insertion).unwrap().pair().vertex(), v2);

            let e1 = v2_insertion;
            let e2 = self.half_edges[e1].prev;

            self.half_edges[h1].next = e1;
            self.half_edges[e1].prev = h1;

            self.half_edges[e2].next = h2;
            self.half_edges[h2].prev = e2;
        } else {
            debug_assert!(self.vertices[v2].hedge.is_null());
            self.vertices[v2].hedge = h2;
        }

        #[cfg(test)]
        self.verify_invarians();

        Some(e)
    }

    pub fn new_face(&mut self, v: &[VertexHandle], data: DataTypes::Face) -> Option<FaceHandle> {
        if v.len() < 2 {
            return None;
        }
        let mut hedges = Vec::new();

        for (v1, v2) in v.iter().circular_tuple_windows() {
            hedges.push(self.find_or_create_half_edge(*v1, *v2)?);

            debug_assert!(
                self.half_edges[*hedges.last().unwrap()].vertex == *v2
            );
        }

        for hedge in &hedges {
            if !self.half_edges[*hedge].face.is_null() {
                return None;
            }
        }

        // Try to make all half edges adjecent
        for (h1, h2) in hedges.iter().circular_tuple_windows() {
            self.make_hedge_adjencent(*h1, *h2)?;
        }

        let face = self.faces.insert(Face {
            hedge: *hedges.first().unwrap(),
            data,
        });

        for hedge in hedges {
            self.half_edges[hedge].face = face
        }

        #[cfg(test)]
        self.verify_invarians();

        Some(face)
    }

    pub fn find_edge(&self, v1: VertexHandle, v2: VertexHandle) -> Option<EdgeHandle> {
        self.find_half_edge(v1, v2)
            .map(|hedge| self.half_edges[hedge].edge)
    }

    pub fn find_half_edge(&self, v1: VertexHandle, v2: VertexHandle) -> Option<HalfEdgeHandle> {
        let vertex = self.vertex(v1)?;

        for hedge in vertex.out_half_edges() {
            if hedge.vertex() == v2 {
                return Some(hedge.handle());
            }
        }
        None
    }

    pub fn find_or_create_half_edge(
        &mut self,
        v1: VertexHandle,
        v2: VertexHandle,
    ) -> Option<HalfEdgeHandle> {
        if let Some(hedge) = self.find_half_edge(v1, v2) {
            Some(hedge)
        } else {
            self.new_edge(v1, v2, Default::default()).map(|e| {
                let edge = self.edge(e).unwrap();
                debug_assert_eq!(edge.hedge().vertex(), v2);
                edge.hedge().handle()
            })
        }
    }

    fn find_free_half_edge(&self, hedge: HalfEdgeHandle) -> Option<HalfEdgeHandle> {
        let mut current = self.half_edge(hedge).unwrap();
        loop {
            if current.face().is_none() {
                return Some(current.handle());
            }

            current = current.pair().next();
            if current == hedge {
                return None;
            }
        }
    }

    fn find_free_half_edge_between(
        &self,
        after: HalfEdgeHandle,
        before: HalfEdgeHandle,
    ) -> Option<HalfEdgeHandle> {
        debug_assert_eq!(self.half_edges[before].vertex, self.half_edges[after].vertex);

        let mut current = self.half_edge(after).unwrap();
        loop {
            if current == before {
                return None;
            }

            if current.face().is_none() {
                return Some(current.handle());
            }
            current = current.next().pair();
        }
    }

    fn make_hedge_adjencent(&mut self, r#in: HalfEdgeHandle, out: HalfEdgeHandle) -> Option<()> {
        let out_pair = self.half_edges[out].pair;
        debug_assert_eq!(
            self.half_edges[r#in].vertex,
            self.half_edges[out_pair].vertex
        );

        if self.half_edges[r#in].next == out {
            return Some(());
        }

        let in_next = self.half_edges[r#in].next;
        let out_prev = self.half_edges[out].prev;

        let free_in = self.find_free_half_edge_between(out_pair, r#in)?;
        let free_in_next = self.half_edges[free_in].next;

        self.half_edges[r#in].next = out;
        self.half_edges[out].prev = r#in;

        self.half_edges[free_in].next = in_next;
        self.half_edges[in_next].prev = free_in;

        self.half_edges[out_prev].next = free_in_next;
        self.half_edges[free_in_next].prev = out_prev;

        Some(())
    }
}
