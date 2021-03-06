#![forbid(unsafe_code)]
//! Implementation for the Half Edge Graph structure
//!
//! # What is a Half Edge Graph?
//! A Half Edge Graph is a efficent way to represent graphes and have constant time _local_ queries.
//! It works by splitting each edge into two _half edges_.
//!
//! Each half edge contains a pointer to its pair (other half), next & prev pointers,
//! and pointers to its vertex, edge and optionaly face.
//!
//! This structure allows very efficent local queries, eg you can in linear time say what
//! a given vertex is connected to, what edges it have, any faces it is part of, and more.
//!
//! The downside is that it can be quite complex to uphold the invariants that make this possible
//! especially when faces are getting involved. So its _quite_ (relative) expansive to construct
//! the graph, but once you have it, it makes it very easy to traverse the graph and do queries fast.
//!
//! @todo a image is worth a thousand words here
//!
//! # Function Set
//! Function set are the recomommended way to traverse the graph,
//! at their core they are a reference to the graph, and a handle for the item.
//!
//! They offer a erogonomic interface come in 3 flavors:
//!  - shared *Fn ([HalfEdgeFn], [VertexFn], [EdgeFn], [FaceFn])
//!  - mutable *FnMut ([HalfEdgeFnMut], [VertexFnMut], [EdgeFnMut], [FaceFnMut])
//!  - data *FnData ([HalfEdgeFnData], [VertexFnData], [EdgeFnData], [FaceFnData])
//!
//! ## Function Set - Shared
//! These provide shared access to the graph structure, and is the recommended way to traverse the graph
//! and gather data.
//!
//! They provide a erogonomic way to traverse the graph, ex:
//!
//! ```no_run
//! # let graph = half_edge_graph::HalfEdgeGraph::<()>::default();
//! # let vertex_handle = half_edge_graph::VertexHandle::default();
//! let vertex = graph.vertex(vertex_handle).unwrap();
//! for edge in vertex.edges() {
//!     // do something with edge - note that edge is a `EdgeFn`
//! }
//! ```
//!
//! ## Function Set - Mutable
//! These provide mutable access to the graph, use if you need to modify the graph. You can use them to
//! traverse the graph, but they are not so erogonomic as the *Fn version, for example if you want to
//! iterate over all vertices in a face you can't use a regular for loop, instead you have to do:
//! ```no_run
//! # let mut graph = half_edge_graph::HalfEdgeGraph::<()>::default();
//! # let face_handle = half_edge_graph::FaceHandle::default();
//!  let mut face = graph.face_mut(face_handle).unwrap();
//!  let mut face_vertices = face.vertices_mut();
//!  while let Some(vertex) = face_vertices.next() {
//!     // do something with the vertex
//!  }
//! ```
//! This will probably improve once GAT's & LendingIterator are stabilized.
//!
//! ## Function Set - Data
//! These are only used when you want to iterate over all items mutable in a graph. Ideally I would
//! like to remove these and use the *FnMut variants instead, but due to implementation details it can't
//! currently be implemented effeciently.
//!
//! Internally the HalfEdgeGraph is using [slotmap::SlotMap] for storage, and in order to implement the iterator
//! efficently we would need a way to get a handle for a given slot, however this is not currently supported, so
//! instead it would have to be implemented by first iterating over all handles, collecting them into a vector
//! and using that vector to create the *FnMut version.
//!
//! Instead I choose to implement it using [slotmap::SlotMap::iter_mut], but this makes it unsound to access any
//! links, and only mutable access to the data can be provided.
use std::fmt::Debug;

use itertools::Itertools;
use slotmap::Key;

mod function_set;
pub use function_set::{
    EdgeFn, EdgeFnMut, FaceFn, FaceFnMut, HalfEdgeFn, HalfEdgeFnMut, VertexFn, VertexFnMut,
};

mod iterators;
pub use iterators::{
    EdgeFaces, EdgeFacesMut, EdgesMut, FaceEdges, FaceEdgesMut, FaceFaces, FaceFacesMut,
    FaceVertices, FaceVerticesMut, FacesMut, HalfEdgesMut, VertexEdges, VertexEdgesMut,
    VertexFaces, VertexFacesMut, VertexInHalfEdges, VertexInHalfEdgesMut, VertexOutHalfEdges,
    VertexOutHalfEdgesMut, VertexVertex, VertexVertexMut, VerticesMut,
};

#[cfg(test)]
mod test;

slotmap::new_key_type!(
    pub struct FaceHandle;
    pub struct EdgeHandle;
    pub struct HalfEdgeHandle;
    pub struct VertexHandle;
);

impl FaceHandle {
    pub fn is_null(&self) -> bool {
        <Self as Key>::is_null(self)
    }
    pub fn null() -> Self {
        <Self as Key>::null()
    }
}
impl EdgeHandle {
    pub fn is_null(&self) -> bool {
        <Self as Key>::is_null(self)
    }
    pub fn null() -> Self {
        <Self as Key>::null()
    }
}
impl HalfEdgeHandle {
    pub fn is_null(&self) -> bool {
        <Self as Key>::is_null(self)
    }
    pub fn null() -> Self {
        <Self as Key>::null()
    }
}
impl VertexHandle {
    pub fn is_null(&self) -> bool {
        <Self as Key>::is_null(self)
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
            .transpose()
            .ok()?;

        let v2_insertion = vertex_2
            .hedge()
            .map(|h| self.find_free_half_edge(h.handle()).ok_or(()))
            .transpose()
            .ok()?;

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

            debug_assert!(self.half_edges[*hedges.last().unwrap()].vertex == *v2);
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
        debug_assert_eq!(
            self.half_edges[before].vertex,
            self.half_edges[after].vertex
        );

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

    pub fn vertex(&self, handle: VertexHandle) -> Option<VertexFn<'_, DataTypes>> {
        if self.vertices.contains_key(handle) {
            Some(VertexFn::new(self, handle))
        } else {
            None
        }
    }

    pub fn vertex_mut(&mut self, handle: VertexHandle) -> Option<VertexFnMut<'_, DataTypes>> {
        if self.vertices.contains_key(handle) {
            Some(VertexFnMut::new(self, handle))
        } else {
            None
        }
    }

    pub fn half_edge(&self, handle: HalfEdgeHandle) -> Option<HalfEdgeFn<'_, DataTypes>> {
        if self.half_edges.contains_key(handle) {
            Some(HalfEdgeFn::new(self, handle))
        } else {
            None
        }
    }

    pub fn half_edge_mut(
        &mut self,
        handle: HalfEdgeHandle,
    ) -> Option<HalfEdgeFnMut<'_, DataTypes>> {
        if self.half_edges.contains_key(handle) {
            Some(HalfEdgeFnMut::new(self, handle))
        } else {
            None
        }
    }

    pub fn edge(&self, handle: EdgeHandle) -> Option<EdgeFn<'_, DataTypes>> {
        if self.edges.contains_key(handle) {
            Some(EdgeFn::new(self, handle))
        } else {
            None
        }
    }

    pub fn edge_mut(&mut self, handle: EdgeHandle) -> Option<EdgeFnMut<'_, DataTypes>> {
        if self.edges.contains_key(handle) {
            Some(EdgeFnMut::new(self, handle))
        } else {
            None
        }
    }

    pub fn face(&self, handle: FaceHandle) -> Option<FaceFn<'_, DataTypes>> {
        if self.faces.contains_key(handle) {
            Some(FaceFn::new(self, handle))
        } else {
            None
        }
    }

    pub fn face_mut(&mut self, handle: FaceHandle) -> Option<FaceFnMut<'_, DataTypes>> {
        if self.faces.contains_key(handle) {
            Some(FaceFnMut::new(self, handle))
        } else {
            None
        }
    }

    pub fn iter_vertices(&self) -> impl Iterator<Item = VertexFn<'_, DataTypes>> {
        self.vertices
            .keys()
            .map(move |handle| VertexFn::new(self, handle))
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = EdgeFn<DataTypes>> {
        self.edges
            .keys()
            .map(move |handle| EdgeFn::new(self, handle))
    }

    pub fn iter_faces(&self) -> impl Iterator<Item = FaceFn<'_, DataTypes>> {
        self.faces
            .keys()
            .map(move |handle| FaceFn::new(self, handle))
    }

    pub fn iter_half_edges(&self) -> impl Iterator<Item = HalfEdgeFn<'_, DataTypes>> {
        self.half_edges
            .keys()
            .map(move |handle| HalfEdgeFn::new(self, handle))
    }

    pub fn iter_vertices_mut(&mut self) -> VerticesMut<'_, DataTypes> {
        VerticesMut::new(self)
    }

    pub fn iter_edges_mut(&mut self) -> EdgesMut<'_, DataTypes> {
        EdgesMut::new(self)
    }

    pub fn iter_faces_mut(&mut self) -> FacesMut<'_, DataTypes> {
        FacesMut::new(self)
    }

    pub fn iter_half_edges_mut(&mut self) -> HalfEdgesMut<'_, DataTypes> {
        HalfEdgesMut::new(self)
    }
}

impl<DataTypes: Data> std::ops::Index<VertexHandle> for HalfEdgeGraph<DataTypes> {
    type Output = DataTypes::Vertex;

    fn index(&self, index: VertexHandle) -> &Self::Output {
        &self.vertices.get(index).unwrap().data
    }
}

impl<DataTypes: Data> std::ops::IndexMut<VertexHandle> for HalfEdgeGraph<DataTypes> {
    fn index_mut(&mut self, index: VertexHandle) -> &mut Self::Output {
        &mut self.vertices.get_mut(index).unwrap().data
    }
}

impl<DataTypes: Data> std::ops::Index<HalfEdgeHandle> for HalfEdgeGraph<DataTypes> {
    type Output = DataTypes::HalfEdge;

    fn index(&self, index: HalfEdgeHandle) -> &Self::Output {
        &self.half_edges.get(index).unwrap().data
    }
}

impl<DataTypes: Data> std::ops::IndexMut<HalfEdgeHandle> for HalfEdgeGraph<DataTypes> {
    fn index_mut(&mut self, index: HalfEdgeHandle) -> &mut Self::Output {
        &mut self.half_edges.get_mut(index).unwrap().data
    }
}

impl<DataTypes: Data> std::ops::Index<EdgeHandle> for HalfEdgeGraph<DataTypes> {
    type Output = DataTypes::Edge;

    fn index(&self, index: EdgeHandle) -> &Self::Output {
        &self.edges.get(index).unwrap().data
    }
}

impl<DataTypes: Data> std::ops::IndexMut<EdgeHandle> for HalfEdgeGraph<DataTypes> {
    fn index_mut(&mut self, index: EdgeHandle) -> &mut Self::Output {
        &mut self.edges.get_mut(index).unwrap().data
    }
}

impl<DataTypes: Data> std::ops::Index<FaceHandle> for HalfEdgeGraph<DataTypes> {
    type Output = DataTypes::Face;

    fn index(&self, index: FaceHandle) -> &Self::Output {
        &self.faces.get(index).unwrap().data
    }
}

impl<DataTypes: Data> std::ops::IndexMut<FaceHandle> for HalfEdgeGraph<DataTypes> {
    fn index_mut(&mut self, index: FaceHandle) -> &mut Self::Output {
        &mut self.faces.get_mut(index).unwrap().data
    }
}
