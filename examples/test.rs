use slotmap::SlotMap;

slotmap::new_key_type! {
    struct VertexHandle;
}

type VertexData = String;

#[derive(Default)]
struct Vertex {
    next: VertexHandle,
    data: VertexData,
}

#[derive(Default)]
struct Graph(SlotMap<VertexHandle, Vertex>);

impl Graph {
    fn iter_vertices(&self) -> impl Iterator<Item = VertexFn> {
        self.0.keys().map(move |h| VertexFn::new(self, h))
    }
}

struct VertexFn<'graph> {
    graph: &'graph Graph,
    handle: VertexHandle,
}

impl<'graph> VertexFn<'graph> {
    fn new(graph: &'graph Graph, handle: VertexHandle) -> Self {
        Self { graph, handle }
    }

    fn next(&self) -> Self {
        Self {
            graph: self.graph,
            handle: self.graph.0[self.handle].next,
        }
    }

    fn data(&self) -> &VertexData {
        &self.graph.0[self.handle].data
    }
}

struct VertexFnMut<'graph> {
    graph: &'graph mut Graph,
    handle: VertexHandle,
}

impl<'graph> VertexFnMut<'graph> {
    fn new(graph: &'graph mut Graph, handle: VertexHandle) -> Self {
        Self { graph, handle }
    }

    fn next<'a: 'graph>(&'a mut self) -> VertexFnMut<'a> {
        let handle = self.graph.0[self.handle].next;
        Self {
            graph: self.graph,
            handle,
        }
    }

    fn data(&mut self) -> &mut VertexData {
        &mut self.graph.0[self.handle].data
    }
}

fn main() {
    let mut graph = Graph::default();

    let h1 = graph.0.insert(Vertex::default());
    let h2 = graph.0.insert(Vertex::default());

    graph.0[h1].next = h2;
    graph.0[h1].data = "Hello".to_string();

    graph.0[h2].next = h1;
    graph.0[h2].data = "World".to_string();

    let v1 = VertexFn::new(&graph, h1);

    dbg!(v1.data());
    dbg!(v1.next().data());

    for v in graph.iter_vertices() {
        dbg!(v.data());
    }

    let mut fn_mut_1 = VertexFnMut::new(&mut graph, h1);

    dbg!(fn_mut_1.next().data());
}
