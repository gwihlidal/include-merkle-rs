extern crate include_merkle;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn write_output(identity: &str, data: &[u8]) {
    // Make sure the output directory exists
    let output_dir = Path::new("./output");
    std::fs::create_dir_all(output_dir).expect("failed to create output directory");

    // These should match
    assert_eq!(include_merkle::compute_identity(&data), identity);

    let path = output_dir.join(&identity);
    let file = File::create(&path).expect("failed to create output file");
    let mut writer = BufWriter::new(file);
    writer.write_all(data).expect("failed to write output file");
    println!("Wrote data out to {:?}", path);
}

fn main() {
    let output_dir = Path::new("./output");
    std::fs::create_dir_all(output_dir).expect("failed to create output directory");

    let working_dir = Path::new("examples/shaders");
    //let entry_points = vec![working_dir.join("BloomExtractAndDownsampleHdrCS.hlsl"), working_dir.join("TemporalBlendCS.hlsl")];
    let entry_points = vec![working_dir.join("BloomExtractAndDownsampleHdrCS.hlsl")];

    println!("---\nFlatten Test\n*");
    for entry_point in &entry_points {
        let entry_path = entry_point.canonicalize().unwrap();

        let mut entry_graph = include_merkle::IncludeNodeGraph::new();
        let entry_node =
            include_merkle::traverse_build(&mut entry_graph, &working_dir, &entry_path, 0);

        include_merkle::traverse_flatten(&mut entry_graph, entry_node);
        let flattened_text = &entry_graph[entry_node].node.flattened;
        let flattened_data = flattened_text.as_bytes();
        let flattened_identity = include_merkle::compute_identity(&flattened_data);
        write_output(&flattened_identity, &flattened_data);
    }

    println!("---\nPatching Test\n*");
    for entry_point in &entry_points {
        let entry_path = entry_point.canonicalize().unwrap();
        let mut entry_graph = include_merkle::IncludeNodeGraph::new();
        let entry_node =
            include_merkle::traverse_build(&mut entry_graph, &working_dir, &entry_path, 0);
        include_merkle::traverse_patch(&mut entry_graph, entry_node);
        include_merkle::graph_to_node_vec(&entry_graph)
            .iter()
            .for_each(|node| {
                let patched_data = node.flattened.as_bytes();
                let patched_identity = include_merkle::compute_identity(&patched_data);
                write_output(&patched_identity, &patched_data);
            });

        if let Some(ref root_node) = include_merkle::get_root_node(&entry_graph) {
            let patched_identity = match root_node.patched_identity {
                Some(ref identity) => &identity,
                None => "INVALID",
            };
            println!("Patched identity of root node: {}", patched_identity);
        }

        println!("\n{}", include_merkle::graph_to_dot(&entry_graph));
        include_merkle::graph_to_stdout(&entry_graph, entry_node).unwrap();
    }
}
