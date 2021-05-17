extern crate petgraph;
extern crate ptree;
#[macro_use]
extern crate lazy_static;
extern crate base58;
extern crate regex;
extern crate sha2;

use encoding::label::encoding_from_whatwg_label;
use encoding::DecoderTrap;
use normalize_line_endings::normalized;
use petgraph::algo::is_cyclic_directed;
use petgraph::dot::Dot;
use petgraph::prelude::*;
use petgraph::visit::Walker;
use ptree::graph::print_graph;
use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::iter::FromIterator;
use std::path::Path;
use std::path::PathBuf;
use log::trace;

pub fn decode_data_as_utf8(byte_str: &[u8], normalize_endings: bool) -> String {
    let result = chardet::detect(&byte_str);
    let encoding = chardet::charset2encoding(&result.0);
    let coder = encoding_from_whatwg_label(encoding);
    if coder.is_some() {
        let utf8_text = coder
            .unwrap()
            .decode(&byte_str, DecoderTrap::Ignore)
            .expect("Error decoding utf-8 data");
        if normalize_endings {
            let normalized_text = String::from_iter(normalized(utf8_text.chars()));
            assert!(!normalized_text.contains('\r'));
            normalized_text
        } else {
            utf8_text
        }
    } else {
        String::new()
    }
}

/// Represents a pattern matched include directive.
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

/// Represents a particular include file.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct IncludeNode {
    /// Canonical path of working directory
    pub working_dir: PathBuf,

    /// Canonical path of include file
    pub include_file: PathBuf,

    /// Original identity of the source (no modifications)
    pub source_identity: Option<String>,

    /// Modified identity of the source (flattened or Merkle replacement)
    pub patched_identity: Option<String>,

    /// Resolved file contents (flattened or patched)
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
    /// Create a new `IncludeNode` from a working directory and a file path.
    pub fn new(working_dir: &Path, include_file: &Path) -> Self {
        IncludeNode {
            working_dir: working_dir.into(),
            include_file: include_file.into(),
            source_identity: None,
            patched_identity: None,
            flattened: String::new(),
        }
    }

    /// Load the contents of the `IncludeNode` backing file and return as a utf8 encoded string.
    pub fn data_as_string(&self, normalize_endings: bool) -> String {
        let data = read_file(&self.include_file);
        if let Ok(ref data) = data {
            decode_data_as_utf8(&data, normalize_endings)
        } else {
            String::new()
        }
    }
}

/// Represents the payload for a node in the graph.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct IncludeNodeWeight {
    /// Include file associated with the graph node.
    pub node: IncludeNode,

    /// Pattern matched include directives for the include file.
    pub includes: Vec<Include>,

    /// Useful identifier for locating the root in raw nodes.
    pub(crate) is_root: bool,
}

pub type IncludeNodeLevel = u32;
pub type IncludeNodeGraph = Graph<IncludeNodeWeight, IncludeNodeLevel>;

/// Compute a Sha256 + Base58 encoded identity for a data slice.
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

/// Traverse the graph in order to construct the structure and meta data.
pub fn traverse_build(
    mut graph: &mut IncludeNodeGraph,
    working_dir: &Path,
    include_file: &Path,
    level: IncludeNodeLevel,
    normalize_endings: bool,
) -> NodeIndex {
    let include_dir = include_file.parent().unwrap();
    let include_node = IncludeNode::new(&working_dir, &include_file);
    let include_text = include_node.data_as_string(normalize_endings);

    // Parse include text and extract all includes.
    let includes = resolve_includes(&include_text, &working_dir, &include_dir);

    let (graph_node, outgoing_nodes) = if graph.node_count() == 0 {
        // Borrowing rules mean we can't add the root node first and share `includes`. Lets only
        // clone one extra time at least (not every iteration).
        let graph_node = graph.add_node(IncludeNodeWeight {
            node: include_node,
            includes: includes.clone(),
            is_root: true,
        });

        let outgoing_nodes = includes
            .iter()
            .map(|ref include| {
                traverse_build(
                    &mut graph,
                    &working_dir,
                    &include.include_path,
                    level + 1,
                    normalize_endings,
                )
            })
            .collect::<Vec<NodeIndex>>();

        (graph_node, outgoing_nodes)
    } else {
        let outgoing_nodes = includes
            .iter()
            .map(|ref include| {
                traverse_build(
                    &mut graph,
                    &working_dir,
                    &include.include_path,
                    level + 1,
                    normalize_endings,
                )
            })
            .collect::<Vec<NodeIndex>>();

        let graph_node = graph.add_node(IncludeNodeWeight {
            node: include_node,
            includes,
            is_root: false,
        });

        (graph_node, outgoing_nodes)
    };

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

/// Traverse the graph in order to patch in Merkle identities for all include directives.
pub fn traverse_patch(graph: &mut IncludeNodeGraph, root_node: NodeIndex, normalize_endings: bool) {
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
            let mut include_text = node.data_as_string(normalize_endings);
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

/// Traverse the graph in order to flatten the text for the root node.
pub fn traverse_flatten(
    graph: &mut IncludeNodeGraph,
    root_node: NodeIndex,
    normalize_endings: bool,
) {
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
            let mut include_text = node.data_as_string(normalize_endings);
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

/// Check if a given path exists on the file system.
pub fn path_exists<P: AsRef<Path>>(path: P) -> bool {
    std::fs::metadata(path.as_ref()).is_ok()
}

/// Convert a path into a string
pub fn path_to_string(path: &Path) -> Option<String> {
    let path_os_str = path.as_os_str();
    if let Some(path_str) = path_os_str.to_str() {
        Some(path_str.to_string())
    } else {
        None
    }
}

/// Strip the working directory prefix from an include file path
pub fn path_strip_base(working_dir: &Path, include_file: &Path) -> PathBuf {
    if let Ok(ref prefix) = working_dir.canonicalize() {
        if let Ok(ref path) = include_file.strip_prefix(&prefix) {
            path.to_path_buf()
        } else {
            include_file.to_path_buf()
        }
    } else {
        include_file.to_path_buf()
    }
}

/// Read a file in its entirety into a byte vector.
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let meta = file.metadata()?;
    let size = meta.len() as usize;
    let mut data = vec![0; size];
    file.read_exact(&mut data)?;
    Ok(data)
}

/// Parse the specified text to extract all relative and absolute include directives.
/// See: https://www.wihlidal.com/blog/pipeline/2018-10-04-parsing-shader-includes/
pub fn parse_includes(input: &str) -> Vec<Include> {
    // Alternate forms:
    // r#"(?m)^*\#include\s+["<]([^">]+)*[">]"#
    // r#"(?m)(^*\#\s*include\s*<([^<>]+)>)|(^\s*\#\s*include\s*"([^"]+)")"#

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

/// Extract resolved include directives from the specified text.
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
            trace!("Include path is invalid: {:?}", include_path);
        }
        exists
    });

    // Sorted references in reverse order to make patching correct, otherwise
    // applying an earlier patch would invalidate the start and end ranges of
    // the later patches.
    includes.sort_by(|a, b| a.range_start.cmp(&b.range_start));
    includes
}

/// Print the graph as a tree view to `stdout`.
pub fn graph_to_stdout(graph: &IncludeNodeGraph, root_node: NodeIndex) -> std::io::Result<()> {
    print_graph(&graph, root_node)
}

/// Get a `dot/graphviz` representation of the graph.
pub fn graph_to_dot(graph: &IncludeNodeGraph) -> String {
    Dot::new(&graph).to_string()
}

/// Get a flat vector of `IncludeNode` instances in no specific order.
pub fn graph_to_node_vec(graph: &IncludeNodeGraph) -> Vec<IncludeNode> {
    graph
        .raw_nodes()
        .iter()
        .map(|node| node.weight.node.clone())
        .collect::<Vec<IncludeNode>>()
}

/// Get the root node payload from the graph
pub fn get_root_node(graph: &IncludeNodeGraph) -> Option<IncludeNode> {
    if let Some(ref node) = graph.raw_nodes().iter().find(|&node| node.weight.is_root) {
        Some(node.weight.node.clone())
    } else {
        None
    }
}
