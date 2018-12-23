extern crate petgraph;
extern crate ptree;
#[macro_use]
extern crate lazy_static;
extern crate base58;
extern crate regex;
extern crate sha2;

use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use petgraph::algo::is_cyclic_directed;
use petgraph::dot::Dot;
use petgraph::prelude::*;
use petgraph::visit::Walker;
use ptree::graph::print_graph;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Include {
    /// Canonical path to included file
    pub include_path: PathBuf,

    /// Start position in text buffer of include directive
    pub range_start: usize,

    /// End position in text buffer of include directive
    pub range_end: usize,

    /// Identifies if the path is relative or absolute
    pub relative_path: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct IncludeNode {
    /// Canonical path of working directory
    pub working_dir: PathBuf,

    /// Canonical path of include file
    pub include_file: PathBuf,

    /// Sha256+Base58 encoded content identity
    pub source_identity: Option<String>,
    pub patched_identity: Option<String>,

    pub flattened: String,
}

impl fmt::Display for IncludeNodeWeight {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let source_identity = match self.node.source_identity {
            Some(ref identity) => &identity,
            None => "INVALID",
        };
        let patched_identity = match self.node.patched_identity {
            Some(ref identity) => &identity,
            None => "INVALID",
        };
        write!(
            f,
            "(s:[{}] p:[{}] f:[{:#?}])",
            &source_identity,
            &patched_identity,
            &self.node.include_file.file_name().unwrap_or_default()
        )
    }
}

impl IncludeNode {
    pub fn new(working_dir: &Path, include_file: &Path) -> Self {
        IncludeNode {
            working_dir: working_dir.into(),
            include_file: include_file.into(),
            source_identity: None,
            patched_identity: None,
            flattened: String::new(),
        }
    }

    pub fn data_as_string(&self) -> String {
        let data = read_file(&self.include_file);
        if let Ok(ref data) = data {
            String::from_utf8_lossy(data).to_string()
        } else {
            String::new()
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct IncludeNodeWeight {
    pub node: IncludeNode,
    pub includes: Vec<Include>,
}

pub type IncludeNodeLevel = u32;
pub type IncludeNodeGraph = Graph<IncludeNodeWeight, IncludeNodeLevel>;

pub fn compute_identity(data: &[u8]) -> String {
    use base58::ToBase58;
    use sha2::{Digest, Sha256};

    // Create a Sha256 object
    let mut hasher = Sha256::default();

    // Write input data
    hasher.input(data);

    // Read hash digest and consume hasher
    hasher.result().to_base58()
}

pub fn traverse_build(
    mut graph: &mut IncludeNodeGraph,
    working_dir: &Path,
    include_file: &Path,
    level: IncludeNodeLevel,
) -> NodeIndex {
    let include_dir = include_file.parent().unwrap();
    let include_node = IncludeNode::new(&working_dir, &include_file);
    let include_text = include_node.data_as_string();

    // Parse include text and extract all includes.
    let includes = resolve_includes(&include_text, &working_dir, &include_dir);

    let outgoing_nodes = includes
        .iter()
        .map(|include| traverse_build(&mut graph, &working_dir, &include.include_path, level + 1))
        .collect::<Vec<NodeIndex>>();

    let graph_node = graph.add_node(IncludeNodeWeight {
        node: include_node,
        includes,
    });

    // Create all edges, and add them to the graph.
    outgoing_nodes.iter().for_each(|outgoing_node| {
        let edge = graph.add_edge(graph_node, *outgoing_node, level);
        // TODO: Cycles are currently unsupported!
        assert!(!is_cyclic_directed(&*graph));
        // This might work (untested):
        if is_cyclic_directed(&*graph) {
            graph.remove_edge(edge);
        }
    });

    graph_node
}

pub fn traverse_patch(graph: &mut IncludeNodeGraph, root_node: NodeIndex) {
    // Visit nodes in a depth-first search, emitting nodes in post-order.
    // We want to evaluate data starting at the leaf nodes (no include directives).
    let dfs_nodes = DfsPostOrder::new(&*graph, root_node)
        .iter(&*graph)
        .collect::<Vec<NodeIndex>>();
    dfs_nodes.iter().for_each(|node_index| {
        let neighbors = graph
            .neighbors_directed(*node_index, Direction::Outgoing)
            .map(|neighbor| {
                let neighbor_weight = &graph[neighbor];
                // All neighbors should have identities at this point
                assert!(neighbor_weight.node.source_identity.is_some());
                assert!(neighbor_weight.node.patched_identity.is_some());
                (
                    neighbor_weight.node.include_file.clone(),
                    neighbor_weight
                        .node
                        .patched_identity
                        .as_ref()
                        .unwrap()
                        .clone(),
                )
            })
            .collect::<Vec<(PathBuf, String)>>();

        if let Some(ref mut node_weight) = graph.node_weight_mut(*node_index) {
            let mut node = &mut node_weight.node;
            let mut include_text = node.data_as_string();
            node.source_identity = Some(compute_identity(&include_text.as_bytes()));
            for (ref include_file, ref patched_identity) in neighbors {
                if let Some(ref include) = node_weight
                    .includes
                    .iter()
                    .find(|&include| &include.include_path == include_file)
                {
                    let patch = format!("#include \"{}\"", patched_identity);
                    include_text.replace_range(include.range_start..include.range_end, &patch);
                }
            }
            node.patched_identity = Some(compute_identity(&include_text.as_bytes()));
            node.flattened = include_text;
        }
    });
}

pub fn traverse_flatten(graph: &mut IncludeNodeGraph, root_node: NodeIndex) {
    // Visit nodes in a depth-first search, emitting nodes in post-order.
    // We want to evaluate data starting at the leaf nodes (no include directives).
    let dfs_nodes = DfsPostOrder::new(&*graph, root_node)
        .iter(&*graph)
        .collect::<Vec<NodeIndex>>();
    dfs_nodes.iter().for_each(|node_index| {
        let neighbors = graph
            .neighbors_directed(*node_index, Direction::Outgoing)
            .map(|neighbor| {
                let neighbor_weight = &graph[neighbor];
                (
                    neighbor_weight.node.include_file.clone(),
                    neighbor_weight.node.flattened.to_owned(),
                )
            })
            .collect::<Vec<(PathBuf, String)>>();

        if let Some(ref mut node_weight) = graph.node_weight_mut(*node_index) {
            let mut node = &mut node_weight.node;
            let mut include_text = node.data_as_string();
            node.source_identity = Some(compute_identity(&include_text.as_bytes()));
            for (ref include_file, ref flattened) in neighbors {
                if let Some(ref include) = node_weight
                    .includes
                    .iter()
                    .find(|&include| &include.include_path == include_file)
                {
                    let patch = format!(
                        "// EMBED-START - {:?}\n{}\n// EMBED-FINISH - {:?}",
                        &include_file, &flattened, &include_file
                    );
                    include_text.replace_range(include.range_start..include.range_end, &patch);
                }
            }
            node.patched_identity = Some(compute_identity(&include_text.as_bytes()));
            node.flattened = include_text;
        }
    });
}

pub fn path_exists<P: AsRef<Path>>(path: P) -> bool {
    std::fs::metadata(path.as_ref()).is_ok()
}

pub fn path_to_string(path: &Path) -> Option<String> {
    let path_os_str = path.as_os_str();
    if let Some(path_str) = path_os_str.to_str() {
        Some(path_str.to_string())
    } else {
        None
    }
}

pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let meta = file.metadata()?;
    let size = meta.len() as usize;
    let mut data = vec![0; size];
    file.read_exact(&mut data)?;
    Ok(data)
}

pub fn parse_includes(input: &str) -> Vec<Include> {
    //r#"(?m)^*\#include\s+["<]([^">]+)*[">]"#
    //r#"(?m)(^*\#\s*include\s*<([^<>]+)>)|(^\s*\#\s*include\s*"([^"]+)")"#

    lazy_static! {
        static ref ABSOLUTE_PATH_REGEX: Regex = Regex::new(r#"(?m)^*\#\s*include\s*<([^<>]+)>"#)
            .expect("failed to compile absolute include path regex");
    }

    lazy_static! {
        static ref RELATIVE_PATH_REGEX: Regex = Regex::new(r#"(?m)^*\#\s*include\s*"([^"]+)""#)
            .expect("failed to compile relative include path regex");
    }

    let mut references: Vec<Include> = Vec::with_capacity(8);

    // Result will be an iterator over tuples containing the start and end indices for each match in the string
    let absolute_results = ABSOLUTE_PATH_REGEX.find_iter(input);
    for absolute_result in absolute_results {
        let range_start = absolute_result.start();
        let range_end = absolute_result.end();
        let range_text = &input[range_start..range_end];
        let range_caps = ABSOLUTE_PATH_REGEX.captures(range_text).unwrap();
        let include_path = range_caps.get(1).map_or("", |m| m.as_str());
        if !include_path.is_empty() {
            references.push(Include {
                include_path: Path::new(include_path).to_path_buf(),
                range_start,
                range_end,
                relative_path: false,
            });
        }
    }

    let relative_results = RELATIVE_PATH_REGEX.find_iter(input);
    for relative_result in relative_results {
        let range_start = relative_result.start();
        let range_end = relative_result.end();
        let range_text = &input[range_start..range_end];
        let range_text = range_text.trim().trim_matches('\n');
        let range_caps = RELATIVE_PATH_REGEX.captures(range_text).unwrap();
        let include_path = range_caps.get(1).map_or("", |m| m.as_str());
        if !include_path.is_empty() {
            references.push(Include {
                include_path: Path::new(include_path).to_path_buf(),
                range_start,
                range_end,
                relative_path: true,
            });
        }
    }

    references
}

pub fn resolve_includes(text: &str, working_dir: &Path, include_dir: &Path) -> Vec<Include> {
    let mut includes = parse_includes(&text);
    for include in &mut includes {
        let parent_path = if include.relative_path {
            include_dir
        } else {
            working_dir
        };

        let full_path = parent_path.join(&include.include_path);
        if let Ok(ref canonicalized) = &full_path.canonicalize() {
            include.include_path = canonicalized.to_path_buf();
        }
    }

    includes.retain(|include| {
        let include_path = Path::new(&include.include_path);
        let exists = path_exists(&include_path);
        if !exists {
            println!("Include path is invalid: {:?}", include_path);
        }
        exists
    });

    includes
}

pub fn graph_to_stdout(graph: &IncludeNodeGraph, root_node: NodeIndex) -> std::io::Result<()> {
    print_graph(&graph, root_node)
}

pub fn graph_to_dot(graph: &IncludeNodeGraph) -> String {
    Dot::new(&graph).to_string()
}

pub fn graph_to_node_vec(graph: &IncludeNodeGraph) -> Vec<IncludeNode> {
    graph
        .raw_nodes()
        .iter()
        .map(|node| node.weight.node.clone())
        .collect::<Vec<IncludeNode>>()
}
