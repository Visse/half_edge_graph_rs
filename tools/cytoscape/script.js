let SHOW_EDGES = false;
let SHOW_PREV = true;

let style = [
    {
        selector: 'node',
        style: {
            'background-color': '#666',
            'label': 'data(id)',
            'font-size': '8px',
            'width': '12px',
            'height': '12px',
        }
    },
    {
        selector: 'edge',
        style: {
            'width': 1,
            'line-color': '#ccc',
            'target-arrow-color': '#ccc',
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier',
            'font-size': '8px',
        }
    },
    {
        selector: '[class="vertex"]',
        style: {
            'background-color': '#F00',
            'label': 'data(id)',
            'width': '20px',
            'height': '20px',
            'font-size': '12px',
        }
    },
    {
        selector: '[class="edge"]',
        style: {
            'label': 'data(id)',
        }
    },
    {
        selector: '[class="half_edge"]',
        style: {
            'background-color': '#00F',
            'label': 'data(id)',
        }
    },
    {
        selector: '[class="hedge_pair"]',
        style: {
            'width': 1,
            // 'label': 'pair',
            'target-arrow-shape': 'none',
            'line-color': '#00F',
        }
    },
    {
        selector: '[class="hedge_next"]',
        style: {
            'width': 1,
            // 'label': 'next',
            'line-color': '#0F0',
            'target-arrow-shape': 'triangle',
            'target-arrow-color': '#0F0',
        }
    },
    {
        selector: '[class="hedge_prev"]',
        style: {
            'width': 1,
            // 'label': 'next',
            'line-color': '#080',
            'target-arrow-shape': 'triangle',
            'target-arrow-color': '#080',
        }
    },
    {
        selector: '[class="hedge_vertex"]',
        style: {
            'width': 1,
            // 'label': 'vertex',
            'line-color': '#F00',
            'target-arrow-shape': 'triangle',
            'target-arrow-color': '#F00',
        }
    },
    {
        selector: '[class="hedge"]',
        style: {
            'width': 1,
            // 'label': 'hedge',
            'line-color': '#FF0',
            'target-arrow-shape': 'triangle',
            'target-arrow-color': '#FF0',
        }
    },
    {
        selector: '[class="vertex_edge"]',
        style: {
            'width': 3,
            'target-arrow-shape': 'none',
            'label': 'data(edge)',
            'font-size': '6px',
        }
    },
];


var cy = cytoscape({
    container: $('#graph'),
    style: style,
});


console.log('Adding vertices');
for(let handle in data.vertices) {
    let vertex = data.vertices[handle];
    cy.add({
        group: 'nodes',
        data: {
            id: handle,
            class: 'vertex'
        }
    });
}

// Add 'virtual' edges between vertices
for(let handle in data.vertices) {
    let vertex = data.vertices[handle];
    if (!vertex.hedge) {
        continue;
    }
    let current = vertex.hedge;
    for(let i=0; i < 20; ++i) { // (max 20 edges, to prevent infinite loop in case of bad data)
        let target = data.half_edges[current].vertex;
        let edge = data.half_edges[current].edge;
        if (handle < target) {
            cy.add({
                group: 'edges',
                data: {
                    class: 'vertex_edge',
                    source: handle,
                    target: target,
                    edge: edge,
                }
            });
        }

        current = data.half_edges[data.half_edges[current].pair].next;

        if (current == vertex.hedge) {
            break;
        }
    }
}


// Add half edges
for(let handle in data.half_edges) {
    let hedge = data.half_edges[handle];

    cy.add({
        group: 'nodes',
        data: {
            id: handle,
            class: 'half_edge',
        }
    });
}

if (SHOW_EDGES) {
    // Add edges
    for(let handle in data.edges) {
        cy.add({
            group: 'nodes',
            data: {
                id: handle,
                class: 'edge',
            }
        });
    }
}

// Add faces
for(let handle in data.faces) {
    cy.add({
        group: 'nodes',
        data: {
            id: handle,
            class: 'face',
        }
    });
}

// Add half edges connections
for(let handle in data.half_edges) {
    let hedge = data.half_edges[handle];

    if (handle < hedge.pair) {
        cy.add({
            group: 'edges',
            data: {
                class: 'hedge_pair',
                source: handle,
                target: hedge.pair
            }
        });
    }
    cy.add({
        group: 'edges',
        data: {
            class: 'hedge_next',
            source: handle,
            target: hedge.next
        }
    });
    if (SHOW_PREV) {
        cy.add({
            group: 'edges',
            data: {
                class: 'hedge_prev',
                source: handle,
                target: hedge.prev
            }
        });
    }

    cy.add({
        group: 'edges',
        data: {
            class: 'hedge_vertex',
            source: handle,
            target: hedge.vertex
        }
    });

    if (SHOW_EDGES) {
        cy.add({
            group: 'edges',
            data: {
                class: 'hedge_edge',
                source: handle,
                target: hedge.edge
            }
        });
    }
}

for(let handle in data.vertices) {
    let vertex = data.vertices[handle];
    if (vertex.hedge) {
        cy.add({
            group: 'edges',
            data: {
                class: 'hedge',
                source: handle,
                target: vertex.hedge
            }
        });
    }
}

for(let handle in data.faces) {
    let face = data.faces[handle];
    cy.add({
        group: 'edges',
        data: {
            class: 'hedge',
            source: handle,
            target: face.hedge
        }
    });
}

// Run a layout of the vertices
cy.$('[class="vertex"], [class="vertex_edge"]').layout({
    name: 'cose',
    idealEdgeLength: 200,
    edgeElasticity: 100,
    nodeRepulsion: 400000,
    animate: false,
    stop: do_layout
}).run()

cy.$('[class="vertex"]').on('drag', do_layout);

// Add half edges

function do_layout()
{
    cy.batch(function() {
        // layout half edges
        for(let handle in data.half_edges) {
            let hedge = data.half_edges[handle];
            let pair = data.half_edges[hedge.pair];
        
            let v1 = Victor.fromObject(cy.$id(hedge.vertex).position());
            let v2 = Victor.fromObject(cy.$id(pair.vertex).position());
        
            let mid = v1.clone().add(v2).multiply(new Victor(0.5, 0.5));
        
            let offset = v1.subtract(v2).normalize().rotateDeg(90).multiply(new Victor(30, 30));
            let position = mid.clone().add(offset);
        
            cy.$id(handle).position(position.toObject());

            if (SHOW_EDGES) {
                // update edge position
                if (handle < hedge.pair) {
                    cy.$id(hedge.edge).position(mid.toObject());
                }
            }
        }
    });
}




/*
cy.layout({
    name: 'preset',
}).run()
*/
// layout: {
//     name: 'cose',
//     idealEdgeLength: 200,
//     nodeOverlap: 20,
//     refresh: 20,
//     fit: true,
//     padding: 30,
//     randomize: false,
//     componentSpacing: 100,
//     nodeRepulsion: 400000,
//     edgeElasticity: 100,
//     nestingFactor: 5,
//     gravity: 80,
//     numIter: 1000,
//     initialTemp: 200,
//     coolingFactor: 0.95,
//     minTemp: 1.0
// }
