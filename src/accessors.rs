use super::{
    Data, EdgeFn, EdgeHandle, FaceFn, FaceHandle, HalfEdgeFn, HalfEdgeGraph, HalfEdgeHandle,
    VertexFn, VertexHandle,
};

impl<DataTypes: Data> HalfEdgeGraph<DataTypes> {
    pub fn vertex(&self, handle: VertexHandle) -> Option<VertexFn<DataTypes>> {
        if self.vertices.contains_key(handle) {
            Some(VertexFn::new(self, handle))
        } else {
            None
        }
    }

    pub fn half_edge(&self, handle: HalfEdgeHandle) -> Option<HalfEdgeFn<DataTypes>> {
        if self.half_edges.contains_key(handle) {
            Some(HalfEdgeFn::new(self, handle))
        } else {
            None
        }
    }

    pub fn edge(&self, handle: EdgeHandle) -> Option<EdgeFn<DataTypes>> {
        if self.edges.contains_key(handle) {
            Some(EdgeFn::new(self, handle))
        } else {
            None
        }
    }

    pub fn face(&self, handle: FaceHandle) -> Option<FaceFn<DataTypes>> {
        if self.faces.contains_key(handle) {
            Some(FaceFn::new(self, handle))
        } else {
            None
        }
    }

    pub fn get_vertex_data(&self, handle: VertexHandle) -> Option<&DataTypes::Vertex> {
        self.vertices.get(handle).map(|v| &v.data)
    }

    pub fn get_half_edge_data(&self, handle: HalfEdgeHandle) -> Option<&DataTypes::HalfEdge> {
        self.half_edges.get(handle).map(|v| &v.data)
    }

    pub fn get_edge_data(&self, handle: EdgeHandle) -> Option<&DataTypes::Edge> {
        self.edges.get(handle).map(|v| &v.data)
    }

    pub fn get_face_data(&self, handle: FaceHandle) -> Option<&DataTypes::Face> {
        self.faces.get(handle).map(|v| &v.data)
    }

    pub fn get_vertex_data_mut(&mut self, handle: VertexHandle) -> Option<&mut DataTypes::Vertex> {
        self.vertices.get_mut(handle).map(|v| &mut v.data)
    }

    pub fn get_half_edge_data_mut(
        &mut self,
        handle: HalfEdgeHandle,
    ) -> Option<&mut DataTypes::HalfEdge> {
        self.half_edges.get_mut(handle).map(|v| &mut v.data)
    }

    pub fn get_edge_data_mut(&mut self, handle: EdgeHandle) -> Option<&mut DataTypes::Edge> {
        self.edges.get_mut(handle).map(|v| &mut v.data)
    }

    pub fn get_face_data_mut(&mut self, handle: FaceHandle) -> Option<&mut DataTypes::Face> {
        self.faces.get_mut(handle).map(|v| &mut v.data)
    }
}

impl<DataTypes: Data> std::ops::Index<VertexHandle> for HalfEdgeGraph<DataTypes> {
    type Output = DataTypes::Vertex;

    fn index(&self, index: VertexHandle) -> &Self::Output {
        &self.vertices[index].data
    }
}

impl<DataTypes: Data> std::ops::Index<HalfEdgeHandle> for HalfEdgeGraph<DataTypes> {
    type Output = DataTypes::HalfEdge;

    fn index(&self, index: HalfEdgeHandle) -> &Self::Output {
        &self.half_edges[index].data
    }
}

impl<DataTypes: Data> std::ops::Index<EdgeHandle> for HalfEdgeGraph<DataTypes> {
    type Output = DataTypes::Edge;

    fn index(&self, index: EdgeHandle) -> &Self::Output {
        &self.edges[index].data
    }
}

impl<DataTypes: Data> std::ops::Index<FaceHandle> for HalfEdgeGraph<DataTypes> {
    type Output = DataTypes::Face;

    fn index(&self, index: FaceHandle) -> &Self::Output {
        &self.faces[index].data
    }
}

impl<DataTypes: Data> std::ops::IndexMut<VertexHandle> for HalfEdgeGraph<DataTypes> {
    fn index_mut(&mut self, index: VertexHandle) -> &mut Self::Output {
        &mut self.vertices[index].data
    }
}

impl<DataTypes: Data> std::ops::IndexMut<HalfEdgeHandle> for HalfEdgeGraph<DataTypes> {
    fn index_mut(&mut self, index: HalfEdgeHandle) -> &mut Self::Output {
        &mut self.half_edges[index].data
    }
}

impl<DataTypes: Data> std::ops::IndexMut<EdgeHandle> for HalfEdgeGraph<DataTypes> {
    fn index_mut(&mut self, index: EdgeHandle) -> &mut Self::Output {
        &mut self.edges[index].data
    }
}

impl<DataTypes: Data> std::ops::IndexMut<FaceHandle> for HalfEdgeGraph<DataTypes> {
    fn index_mut(&mut self, index: FaceHandle) -> &mut Self::Output {
        &mut self.faces[index].data
    }
}
