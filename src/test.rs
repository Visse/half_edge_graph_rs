#![allow(unused)]

type HalfEdgeGraph = super::HalfEdgeGraph<()>;
use itertools::Itertools;

use super::{EdgeHandle, FaceHandle, VertexHandle};

use std::collections::{HashMap, HashSet};

impl<DataTypes: super::Data> super::HalfEdgeGraph<DataTypes> {
    #[allow(unused)]
    pub fn dump_to_cytoscape(&self) {
        println!("var data = {{");
        println!("  vertices: {{");
        for (handle, data) in &self.vertices {
            if data.hedge.is_null() {
                println!("    '{:?}': {{hedge: null}},", handle);
            } else {
                println!("    '{:?}': {{hedge: '{:?}'}},", handle, data.hedge);
            }
        }
        println!("  }},");

        println!("  edges: {{");
        for (handle, data) in &self.edges {
            println!("    '{:?}': {{hedge: '{:?}'}},", handle, data.hedge);
        }
        println!("  }},");

        println!("  faces: {{");
        for (handle, data) in &self.faces {
            println!("    '{:?}': {{hedge: '{:?}'}},", handle, data.hedge);
        }
        println!("  }},");

        println!("  half_edges: {{");
        for (handle, data) in &self.half_edges {
            println!(
                "    '{:?}': {{pair: '{:?}', next: '{:?}', prev: '{:?}', vertex: '{:?}', edge: '{:?}', face: '{:?}'}},",
                handle, data.pair, data.next, data.prev, data.vertex, data.edge, data.face
            );
        }
        println!("  }},");
        println!("}};");
    }

    pub fn verify_invarians(&self) {
        for hedge in self.iter_half_edges() {
            assert!(hedge == hedge.pair().pair());
            assert!(hedge == hedge.next().prev());
            assert!(hedge.face() == hedge.next().face());
            assert!(hedge.edge() == hedge.pair().edge());
            assert!(hedge.vertex() != hedge.pair().vertex());
            assert!(hedge.vertex() == hedge.next().pair().vertex());
        }
    }
}

fn check_mesh(
    mesh: &HalfEdgeGraph,
    vertices: &HashSet<VertexHandle>,
    edges: &HashMap<EdgeHandle, (VertexHandle, VertexHandle)>,
    faces: &HashMap<FaceHandle, Vec<VertexHandle>>,
) {
    mesh.verify_invarians();

    // Create auxiliary mappings
    let vertex_edges = {
        let mut mapping: HashMap<VertexHandle, Vec<EdgeHandle>> =
            vertices.iter().map(|v| (*v, Vec::new())).collect();

        for (&edge, &(v1, v2)) in edges {
            mapping.entry(v1).or_insert_with(Vec::new).push(edge);
            mapping.entry(v2).or_insert_with(Vec::new).push(edge);
        }

        mapping
    };
    let vertex_vertex = {
        let mut mapping: HashMap<VertexHandle, Vec<VertexHandle>> =
            vertices.iter().map(|v| (*v, Vec::new())).collect();

        for &(v1, v2) in edges.values() {
            mapping.entry(v1).or_insert_with(Vec::new).push(v2);
            mapping.entry(v2).or_insert_with(Vec::new).push(v1);
        }

        mapping
    };
    let vertex_faces = {
        let mut mapping: HashMap<VertexHandle, Vec<FaceHandle>> =
            vertices.iter().map(|v| (*v, Vec::new())).collect();

        for (&face, vertices) in faces {
            for vertex in vertices {
                mapping.get_mut(vertex).unwrap().push(face);
            }
        }
        mapping
    };

    let edge_lookup = {
        let mut mapping = HashMap::new();
        for (edge, (v1, v2)) in edges {
            mapping.insert((*v1, *v2), *edge);
            mapping.insert((*v2, *v1), *edge);
        }
        mapping
    };

    let (edge_faces, face_edges) = {
        let mut face_mapping: HashMap<FaceHandle, Vec<EdgeHandle>> =
            faces.keys().map(|h| (*h, Vec::new())).collect();
        let mut edge_mapping: HashMap<EdgeHandle, Vec<FaceHandle>> =
            edges.keys().map(|h| (*h, Vec::new())).collect();

        for (face, vertices) in faces {
            for (v1, v2) in vertices
                .iter()
                .chain(std::iter::once(vertices.first().unwrap()))
                .tuple_windows()
            {
                let edge = edge_lookup[&(*v1, *v2)];
                face_mapping.get_mut(face).unwrap().push(edge);
                edge_mapping.get_mut(&edge).unwrap().push(*face);
            }
        }

        (edge_mapping, face_mapping)
    };

    // Make sure we can iterate over all items once
    {
        // create a copy, so we can remove visited items
        let mut vertices = (*vertices).clone();
        let mut edges = (*edges).clone();
        let mut faces = (*faces).clone();

        for vertex in mesh.iter_vertices() {
            assert!(
                vertices.contains(&vertex.handle()),
                "Unknown vertex {:?}",
                vertex.handle()
            );
            vertices.remove(&vertex.handle());
        }
        for edge in mesh.iter_edges() {
            assert!(
                edges.contains_key(&edge.handle()),
                "Unknown edge {:?}",
                edge.handle()
            );
            edges.remove(&edge.handle());
        }
        for face in mesh.iter_faces() {
            assert!(
                faces.contains_key(&face.handle()),
                "Unknown face {:?}",
                face.handle()
            );
            faces.remove(&face.handle());
        }
    }

    // check half edges
    for hedge in mesh.iter_half_edges() {
        assert_ne!(hedge, hedge.pair());
        assert_eq!(hedge, hedge.pair().pair());
        assert_eq!(hedge, hedge.next().prev());
        assert_eq!(hedge.edge(), hedge.pair().edge());
        assert_eq!(hedge.face(), hedge.next().face());
        assert_ne!(hedge.vertex(), hedge.pair().vertex());
    }

    // verify vertexes
    for vertex in mesh.iter_vertices() {
        let mut edges: HashSet<_> = vertex_edges[&vertex.handle()].iter().collect();
        for in_hedge in vertex.in_half_edges() {
            assert_eq!(in_hedge.vertex(), vertex);
            let edge = in_hedge.edge().handle();
            assert!(edges.contains(&edge));
            edges.remove(&edge);
        }
        assert!(edges.is_empty());

        let mut vertices: HashSet<_> = vertex_vertex[&vertex.handle()].iter().collect();
        for out_hedge in vertex.out_half_edges() {
            assert_eq!(out_hedge.pair().vertex(), vertex);
            // check that an edge actually exists
            let target = out_hedge.vertex().handle();
            assert!(vertices.remove(&target));
        }
        assert!(vertices.is_empty());

        let mut vertices: HashSet<_> = vertex_vertex[&vertex.handle()].iter().collect();
        for vertex in vertex.vertices() {
            assert!(vertices.remove(&vertex.handle()));
        }
        assert!(vertices.is_empty());


        let mut edges: HashSet<_> = vertex_edges[&vertex.handle()].iter().collect();
        for edge in vertex.edges() {
            let edge = edge.handle();
            assert!(edges.remove(&edge));
        }
        assert!(
            edges.is_empty(),
            "Not all edges where visited by vertex.edges()"
        );

        let mut faces: HashSet<_> = vertex_faces[&vertex.handle()].iter().collect();
        for face in vertex.faces() {
            let face = face.handle();
            assert!(faces.remove(&face));
        }
    }

    // verify edges
    for edge in mesh.iter_edges() {
        let [v1, v2] = edge.vertices();
        let v1 = v1.handle();
        let v2 = v2.handle();

        assert!(edge_lookup[&(v1, v2)] == edge);

        let mut faces: HashSet<_> = edge_faces[&edge.handle()].iter().collect();
        for face in edge.faces() {
            let face = face.handle();
            assert!(faces.remove(&face));
        }
        assert!(faces.is_empty());
    }

    // verify faces
    for face in mesh.iter_faces() {
        let mut vertices: HashSet<VertexHandle> = faces[&face.handle()].iter().cloned().collect();

        for v in face.vertices() {
            assert!(vertices.remove(&v.handle()));
        }
        assert!(vertices.is_empty());

        let mut edges: HashSet<EdgeHandle> = face_edges[&face.handle()].iter().cloned().collect();

        for e in face.edges() {
            assert!(edges.remove(&e.handle()));
        }
        assert!(edges.is_empty());
    }
}

#[test]
fn simple_mesh_edges() {
    /*
     * mesh layout:
     *        e1
     *     v1 --- v2        v5
     *      |    /
     *   e3 |   / e2
     *      |  /
     *       v3 ----- v4
     *            e4
     */
    let mut mesh = HalfEdgeGraph::default();

    let v1 = mesh.new_vertex(());
    let v2 = mesh.new_vertex(());
    let v3 = mesh.new_vertex(());
    let v4 = mesh.new_vertex(());
    let v5 = mesh.new_vertex(());

    let e1 = mesh.new_edge(v1, v2, ()).expect("Failed to create edge");

    assert!(
        mesh.new_edge(v1, v2, ()).is_none(),
        "Two vertices can only have max one edge between them"
    );
    // Test the other way around
    assert!(
        mesh.new_edge(v2, v1, ()).is_none(),
        "Two vertices can only have max one edge between them"
    );

    let e2 = mesh.new_edge(v2, v3, ()).expect("Failed to create edge");
    let e3 = mesh.new_edge(v3, v1, ()).expect("Failed to create edge");
    let e4 = mesh.new_edge(v3, v4, ()).expect("Failed to create edge");

    check_mesh(
        &mesh,
        &vec![v1, v2, v3, v4, v5].into_iter().collect(),
        &vec![
            (e1, (v1, v2)),
            (e2, (v2, v3)),
            (e3, (v3, v1)),
            (e4, (v3, v4)),
        ]
        .into_iter()
        .collect(),
        &vec![].into_iter().collect(),
    );
}

#[test]
fn simple_mesh_face() {
    /*
     * mesh layout:
     *        e1
     *     v1 --- v2        v5
     *      | f1 /  \
     *   e3 |   / e2 \ e5
     *      |  /  f2  \
     *       v3 ----- v4
     *            e4
     */
    let mut mesh = HalfEdgeGraph::default();

    let v1 = mesh.new_vertex(());
    let v2 = mesh.new_vertex(());
    let v3 = mesh.new_vertex(());
    let v4 = mesh.new_vertex(());
    let v5 = mesh.new_vertex(());

    let f1 = mesh
        .new_face(&[v1, v2, v3], ())
        .expect("Failed to create face");

    let f2 = mesh
        .new_face(&[v2, v4, v3], ())
        .expect("Failed to create face");

    let e1 = mesh.find_edge(v1, v2).expect("Failed to find edge");
    let e2 = mesh.find_edge(v2, v3).expect("Failed to find edge");
    let e3 = mesh.find_edge(v1, v3).expect("Failed to find edge");
    let e4 = mesh.find_edge(v3, v4).expect("Failed to find edge");
    let e5 = mesh.find_edge(v4, v2).expect("Failed to find edge");

    // let e5 = mesh

    check_mesh(
        &mesh,
        &vec![v1, v2, v3, v4, v5].into_iter().collect(),
        &vec![
            (e1, (v1, v2)),
            (e2, (v2, v3)),
            (e3, (v3, v1)),
            (e4, (v3, v4)),
            (e5, (v4, v2)),
        ]
        .into_iter()
        .collect(),
        &vec![(f1, vec![v1, v2, v3]), (f2, vec![v2, v4, v3])]
            .into_iter()
            .collect(),
    );
}

fn calc_face_edges(
    mesh: &HalfEdgeGraph,
    vertices: &[VertexHandle],
) -> Vec<(EdgeHandle, (VertexHandle, VertexHandle))> {
    vertices
        .iter()
        .chain(std::iter::once(&vertices[0]))
        .tuple_windows()
        .map(|(v1, v2)| {
            let edge = mesh.find_edge(*v1, *v2).unwrap();
            (edge, (*v1, *v2))
        })
        .collect()
}

#[test]
fn mesh_1() {
    let mut mesh = HalfEdgeGraph::default();

    let v1 = mesh.new_vertex(());
    let v2 = mesh.new_vertex(());
    let v3 = mesh.new_vertex(());
    let v4 = mesh.new_vertex(());
    let v5 = mesh.new_vertex(());
    let v6 = mesh.new_vertex(());

    let f1 = mesh
        .new_face(&[v1, v2, v3, v4], ())
        .expect("Failed to create face");
    let f2 = mesh
        .new_face(&[v2, v1, v5, v6], ())
        .expect("Failed to create face");

    let f1_edges = calc_face_edges(&mesh, &[v1, v2, v3, v4]);
    let f2_edges = calc_face_edges(&mesh, &[v2, v1, v5, v6]);
    let edges = f1_edges.iter().chain(f2_edges.iter()).cloned().collect();

    check_mesh(
        &mesh,
        &[v1, v2, v3, v4, v5, v6].iter().copied().collect(),
        &edges,
        &[(f1, vec![v1, v2, v3, v4]), (f2, vec![v2, v1, v5, v6])]
            .iter()
            .cloned()
            .collect(),
    );
}


macro_rules! test_mesh {
    (fn $name:ident() {
        $($props:tt)+
    }) => {
        #[allow(unused_mut)]
        #[test]
        fn $name() {
            let mut mesh = HalfEdgeGraph::default();

            let mut vertices = HashSet::new();
            let mut edges = HashMap::new();
            let mut faces = HashMap::new();

            test_mesh!(__impl(mesh, vertices, edges, faces) $($props)+);

            check_mesh(
                &mesh,
                &vertices,
                &edges,
                &faces
            );
        }
    };

    (__impl($mesh:ident, $vertices:ident, $edges:ident, $faces:ident) ) => {};

    (__impl($mesh:ident, $vertices:ident, $edges:ident, $faces:ident) 
        vertex $name:ident; $($rest:tt)*
    ) => {
        println!(stringify!(Creating vertex  $name));

        let $name = $mesh.new_vertex(());
        $vertices.insert($name);

        test_mesh!(__impl($mesh, $vertices, $edges, $faces) $($rest)*);
    };

    (__impl($mesh:ident, $vertices:ident, $edges:ident, $faces:ident) 
        edge $name:ident ($v1:ident -> $v2:ident); $($rest:tt)*
    ) => {
        println!(stringify!(Creating edge  $name  ($v1 -> $v2)));
        let $name = $mesh.new_edge($v1, $v2, ()).expect("Failed to create edge");
        $edges.insert($name, ($v1, $v2));

        tes
        t_mesh!(__impl($mesh, $vertices, $edges, $faces) $($rest)*);
    };
    (__impl($mesh:ident, $vertices:ident, $edges:ident, $faces:ident) 
        face $name:ident ($($v:ident) -> +); $($rest:tt)*
    ) => {
        println!(stringify!(Creating face  $name  ( $($v) -> + )));

        let vertices = vec![$($v,)+];
        let $name = $mesh.new_face(
            &vertices, ()).expect("Failed to create face");

        for (&v1, &v2) in vertices.iter().circular_tuple_windows() {
            let edge = $mesh.find_edge(v1, v2).expect("Failed to find edge");
            $edges.insert(edge, (v1, v2));
        }

        $faces.insert($name, vertices);

        test_mesh!(__impl($mesh, $vertices, $edges, $faces) $($rest)*);
    };
}


test_mesh! {
    fn mesh_grid() {
        vertex v1;
        vertex v2;
        vertex v3;
        vertex v4;
        vertex v5;
        vertex v6;
        vertex v7;
        vertex v8;
        vertex v9;

        face f1 (v1 -> v2 -> v5 -> v4);
        face f2 (v4 -> v5 -> v8 -> v7);
        face f3 (v2 -> v3 -> v6 -> v5);
        face f3 (v5 -> v6 -> v9 -> v8);
        face f4 (v1 -> v4 -> v7 -> v8 -> v9 -> v6 -> v3 -> v2);
    }
}



test_mesh! {
    fn mesh_hexagon() {
        vertex v1;
        vertex v2;
        vertex v3;
        vertex v4;
        vertex v5;
        vertex v6;
        vertex v7;
        vertex v8;
        vertex v9;

        face f1 (v1 -> v2 -> v3);
        face f3 (v1 -> v4 -> v5);
        face f5 (v1 -> v6 -> v7);
        face f7 (v1 -> v8 -> v9);

        face f2 (v1 -> v3 -> v4);
        face f4 (v1 -> v5 -> v6);
        face f6 (v1 -> v7 -> v8);
        face f8 (v1 -> v9 -> v2);

        face f9 (v9 -> v8 -> v6 -> v5 -> v4 -> v3 -> v2);
    }
}

test_mesh! {
    fn mesh_2() {
        vertex v1;
        vertex v2;
        vertex v3;
        vertex v4;
        vertex v5;
        vertex v6;
        vertex v7;
        vertex v8;

        face f1 (v1 -> v2 -> v3);
        face f2 (v4 -> v5 -> v1);
        face f3 (v1 -> v6 -> v4);
        face f4 (v7 -> v8 -> v4);
    }
}