#[derive(Default)]
struct Data {
    value: i32,
}

impl half_edge_graph::Data for Data {
    type Face = Data;
    type Edge = Data;
    type HalfEdge = Data;
    type Vertex = Data;
}

type HalfEdgeGraph = half_edge_graph::HalfEdgeGraph<Data>;

fn main() {
    let mut graph = HalfEdgeGraph::default();

    let v1 = graph.new_vertex(Default::default());
    let v2 = graph.new_vertex(Default::default());

    let e = graph.new_edge(v1, v2, Default::default()).unwrap();

    let mut vertex = graph.vertex_mut(v1).unwrap();
    let mut edge_iter = vertex.edges_mut();
    edge_iter.next().unwrap().value = 5;

    vertex.vertices_mut().next().unwrap().value = 10;

    assert_eq!(graph.vertex(v1).unwrap().value, 0);
    assert_eq!(graph.vertex(v2).unwrap().value, 10);
    assert_eq!(graph.edge(e).unwrap().value, 5);
}
