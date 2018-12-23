extern crate include_merkle;
use std::path::Path;

fn main() {
    let working_dir = Path::new("data");
    let entry_points = vec![Path::new("examples/shaders/BloomExtractAndDownsampleHdrCS.hlsl"), Path::new("examples/shaders/TemporalBlendCS.hlsl")];
    
    println!("---\nPatching Test\n---");
    for entry_point in &entry_points {
        let entry_path = entry_point.canonicalize().unwrap();
        let mut entry_graph = include_merkle::IncludeNodeGraph::new();
        let entry_node = include_merkle::traverse_build(&mut entry_graph, &working_dir, &entry_path, 0);
        include_merkle::traverse_patch(&mut entry_graph, entry_node);

        println!("{}", include_merkle::graph_to_dot(&entry_graph));
        include_merkle::graph_to_stdout(&entry_graph, entry_node).unwrap();
    }

    println!("---\nFlatten Test\n---");
    for entry_point in &entry_points {
        let entry_path = entry_point.canonicalize().unwrap();
        let mut entry_graph = include_merkle::IncludeNodeGraph::new();
        let entry_node = include_merkle::traverse_build(&mut entry_graph, &working_dir, &entry_path, 0);
        include_merkle::traverse_flatten(&mut entry_graph, entry_node);
        let flattened = &entry_graph[entry_node].node.flattened;
        println!("Flattened Source:\n---\n{}", &flattened);
    }
}
