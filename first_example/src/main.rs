//Std stuff
// - // SECTION - Imports
use std::fs::File;
use std::io::BufReader;
use std::io::Error;
use std::collections::HashMap;
use std::ops::Deref;

//Grapher
use grapher::renderer::Renderer;
use grapher::simulator::SimulatorBuilder;

//Petgraph
use petgraph::graph;
use petgraph::visit::NodeRef;
use petgraph::Graph;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;

//Serde
//use serde_json::{Result, Value};
use serde::{Deserialize, Serialize};
// !SECTION

// * ||||||||||||||||||||||||||| */
// * ||||||||  STRUCTS  |||||||| */
// * ||||||||||||||||||||||||||| */
// - // SECTION - Structs

// & Main Struct (Head of the JSON)
#[derive(Serialize, Deserialize, Debug)]
struct OwlToWovlJSON {
    _comment:               Option<String>,
    header:                 Header,
    namespace:              Option<Vec<String>>,
    metrics:                Option<Metrics>,
    class:                  Vec<Class>,
    classAttribute:         Vec<ClassAttribute>,
    property:               Vec<Property>,
    propertyAttribute:      Vec<PropertyAttribute>,
}

// & Level 1
#[derive(Serialize, Deserialize, Debug)]
struct Header {
    languages:              Vec<String>,
    baseIris:               Option<Vec<String>>,
    title:                  Option<Label>,
    iri:                    Option<String>,
    version:                Option<String>,
    author:                 Option<Vec<String>>,
    description:            Option<Label>,
    labels:                 Option<Label>,
    other:                  Option<Other>,
    prefixList:             Option<PrefixList>
}
#[derive(Serialize, Deserialize, Debug)]
struct Metrics {
    classCount:             u32,
    objectPropertyCount:    u32,
    datatypePropertyCount:  u32,
    individualCount:        u32,
}
#[derive(Serialize, Deserialize, Debug)]
struct Class {
    id:                     String,
    r#type:                 String,
    label:                  Option<String>,
    intersection:           Option<Vec<String>>,
    union:                  Option<Vec<String>>,
    disjointUnion:          Option<Vec<String>>,
    complement:             Option<Vec<String>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ClassAttribute {
    id:                     String,
    iri:                    Option<String>,
    baseIri:                Option<String>,
    instances:              Option<u32>,
    individuals:            Option<Vec<Individuals>>,
    annotations:            Option<Annotations>,
    label:                  Option<Label>,
    comment:                Option<Label>,
    attributes:             Option<Vec<String>>,
    superClasses:           Option<Vec<String>>,
    subClasses:             Option<Vec<String>>,
    complement:             Option<Vec<String>>,
    union:                  Option<Vec<String>>,
    intersection:           Option<Vec<String>>,
    equivalent:             Option<Vec<String>>,
    disjointUnion:          Option<Vec<String>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Property {
    id:                     String,
    r#type:                 String,
}
#[derive(Serialize, Deserialize, Debug)]
struct PropertyAttribute {
    iri:                    Option<String>,
    inverse:                Option<String>,
    baseIri:                Option<String>,
    range:                  String,
    annotations:            Option<Annotations>,
    label:                  Option<Label>,
    superproperty:          Option<Vec<String>>,
    domain:                 String,
    subproperty:            Option<Vec<String>>,
    comment:                Option<Label>,
    attributes:             Option<Vec<String>>,
    id:                     String,
}

// & Level 2
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Label {
    // ^ Struct for both labels, comments, title and description
    #[serde(rename = "IRI-based")]
    IRI_based:              Option<String>,
    iriBased:               Option<String>,
    undefined:              Option<String>,
    en:                     Option<String>,
    de:                     Option<String>,
    fr:                     Option<String>,
    es:                     Option<String>,
}
impl Label {
    fn new() -> Label {
        Label {
            IRI_based: None,
            iriBased: None,
            undefined: None,
            en: None,
            de: None,
            fr: None,
            es: None,
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct Other {
    licence:                Option<Vec<ILVT>>,
    creator:                Option<Vec<ILVT>>,
    versionInfo:            Option<Vec<ILVT>>,
    title:                  Option<Vec<ILVT>>,
    issued:                 Option<Vec<ILVT>>,
    seeAlso:                Option<Vec<ILVT>>,
    homepage:               Option<Vec<ILVT>>,
    depiction:              Option<Vec<ILVT>>,
    priorVersion:           Option<Vec<ILVT>>,
    date:                   Option<Vec<ILVT>>,
    contributor:            Option<Vec<ILVT>>,
    incompatibleWith:       Option<Vec<ILVT>>,
    rights:                 Option<Vec<ILVT>>,
    backwardCompatibleWith: Option<Vec<ILVT>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Annotations {
    isDefinedBy:            Option<Vec<ILVT>>,
    versionInfo:            Option<Vec<ILVT>>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Individuals {
    iri:                    String,
    baseIri:                String,
    description:            Option<Label>,
    labels:                 Option<Label>,
}
#[derive(Serialize, Deserialize, Debug)]
struct PrefixList {
    owl:                    Option<String>,
    rdf:                    Option<String>,
    wot:                    Option<String>,
    xsd:                    Option<String>,
    dc:                     Option<String>,
    xml:                    Option<String>,
    vs:                     Option<String>,
    foaf:                   Option<String>,
    rdfs:                   Option<String>,
}

// & Level 3
#[derive(Serialize, Deserialize, Debug)]
struct ILVT {
    identifier:                 String,
    language:                   String,
    value:                      String,
    r#type:                     String,
}

// ^ Custom struct for the edges and nodes of the graph
#[derive(Debug)]
struct Edge {
    domain:                     NodeIndex,
    range:                      NodeIndex,
    r#type:                     EdgeType,
}
#[derive(Debug)]
struct Node {
    id:                         String,
    r#type:                     String,
    label:                      Label,
    edges:                      Option<Vec<Edge>>,
}
// !SECTION

// - // SECTION - Enums
#[derive(Debug)]
enum EdgeType {
    SubClass,
    SuperClass,
    EquivalentClass,
    DisjointUnion,
    Complement,
    Union,
    Intersection,
    Property(Label),
}
// !SECTION

// - // SECTION - Functions
fn main() {

    // * Read the JSON file */
    let file = File::open("./src/foaf.json").unwrap();
    let reader = BufReader::new(file);
    let graph_struct: OwlToWovlJSON = serde_json::from_reader(reader).unwrap();
    
    // * Create the graph */
    let mut graph = Graph::<String, Edge>::new();

    //Hashmap is necessary to store the nodes references with their ids
    let mut node_hashmap = HashMap::new();


    for node in &graph_struct.class {
        let node_id = node.id.clone();
        let opt_index = node_hashmap.get(&node.id);
        match opt_index {
            Some(&x) => {},
            None => {
                // TODO 'Equivalent' and 'Thing' should not be generated as nodes without knowing wether they are connected to something
                node_hashmap.insert(node_id, graph.add_node(node.id.clone()));
            }
        }
    }

    for edge in &graph_struct.propertyAttribute {
        let domain;
        let range;
        let label;

        let dom_opt = node_hashmap.get(&edge.domain.clone());
        match dom_opt {
            Some(&x) => {
                domain = x;
            }
            None => {
                println!("Domain does not exist, skipping edge from {:?}", edge.domain.clone());
                continue;
            }
        }

        let ran_opt = node_hashmap.get(&edge.range.clone());
        match ran_opt {
            Some(&x) => {
                range = x;
            }
            None => {
                println!("Range does not exist, skipping edge to {:?}", edge.range.clone());
                continue;
            }
        }

        match edge.label.clone() {
            Some(edge_label) => {
                label = edge_label.clone();
            }
            None => {
                println!("Label does not exist, creating empty label");
                label = Label::new();
            }
            
        }
        
        let property = Edge {
            domain:     domain,
            range:      range,
            r#type:     EdgeType::Property(label),
        };

        graph.add_edge(domain, range, property);
    }









    // ! temporarily commented out
    // !
    // !for edge in &graph_struct.propertyAttribute {
    // !    let domain_node_id = edge.domain.clone();
    // !    let range_node_id = edge.range.clone();
    // !    let dom_index;
    // !    let ran_index;
    // !
    // !    // * Check if the node already exists */
    // !    // * If it does, get the index        */
    // !    // * If it doesn't, create the node   */
    // !    let opt_dm_index = node_hashmap.get(&domain_node_id);
    // !    match opt_dm_index {
    // !        Some(&x) => {
    // !            dom_index = x;
    // !        },
    // !        None => {
    // !            println!("WARNING! Node does not exist, creating node: {:?}", edge.domain.clone());
    // !            dom_index = graph.add_node(edge.domain.clone());
    // !            node_hashmap.insert(domain_node_id, dom_index);
    // !        }
    // !    }
    // !    
    // !    let opt_rn_index = node_hashmap.get(&edge.range);
    // !    match opt_rn_index {
    // !        Some(&x) => {
    // !            ran_index = x;
    // !        },
    // !        None => {
    // !            println!("WARNING! Node does not exist, creating node: {:?}", edge.range.clone(), );
    // !            ran_index = graph.add_node(edge.range.clone());
    // !            node_hashmap.insert(range_node_id, ran_index);
    // !        }
    // !    }
    // !    graph.add_edge(dom_index, ran_index, ());
    // !}
    // !
    // !for attr in &graph_struct.classAttribute {
    // !    let attr_id = attr.id.clone();
    // !    let opt_index = node_hashmap.get(&attr.id);
    // !
    // !    let dom_index;
    // !    match opt_index {
    // !        Some(&x) => {
    // !            dom_index = x;
    // !        },
    // !        None => {
    // !            continue;
    // !            //println!("WARNING! Node does not exist, creating node: {:?}", attr.id);
    // !            //dom_index = graph.add_node(attr.id.clone());
    // !            //hashmap.insert(attr_id, dom_index);
    // !        }
    // !    }
    // !    make_edges(dom_index, attr.superClasses.clone(), &mut graph, &mut node_hashmap);
    // !    make_edges(dom_index, attr.subClasses.clone(), &mut graph, &mut node_hashmap);
    // !    make_edges(dom_index, attr.complement.clone(), &mut graph, &mut node_hashmap);
    // !    make_edges(dom_index, attr.union.clone(), &mut graph, &mut node_hashmap);
    // !    make_edges(dom_index, attr.intersection.clone(), &mut graph, &mut node_hashmap);
    // !    make_edges(dom_index, attr.disjointUnion.clone(), &mut graph, &mut node_hashmap);
    // !    // ? Used in a different implementation and should not construct edges
    // !    //make_edges(dom_index, attr.equivalent.clone(), &mut graph, &mut hashmap);
    // !}
    
    // ! Debugging stuff
    println!("{:?}", Dot::new(&graph));
    // * println!("Isolated nodes in the file are: {:?}", isolated_nodes(&graph_struct));

    // * Configure the simulator */
    let simulator = SimulatorBuilder::new()
        .delta_time(0.005)
        .freeze_threshold(-1.0)
        .build(graph.into());

    // * Start the renderer */
    let renderer = Renderer::new(simulator);
    renderer.create_window();
}

fn make_edges (domain: NodeIndex, opt_vector: Option<Vec<String>>, graph: &mut Graph<String, ()>, hashmap: &mut HashMap<String, NodeIndex>) {
    match opt_vector {

        Some(vector) => {

            for id_string in vector {
                
                let opt_index = hashmap.get(&id_string);
                match opt_index {

                    Some(ran_index) => {
                        graph.add_edge(domain, *ran_index, ());
                    }
                    None => {
                        println!("WARNING! Node does not exist, creating node: {:?}", id_string);
                        let ran_index = graph.add_node(id_string.clone());
                        hashmap.insert(id_string, ran_index);
                        graph.add_edge(domain, ran_index, ());
                    }
                }
            }
        },
        None => {}
    }
}

fn isolated_nodes (json_struct: &OwlToWovlJSON) -> Vec<String> {
    let mut isolated_nodes = Vec::new();
    'node_iter: for node in &json_struct.class {
        for edge in &json_struct.propertyAttribute {
            if edge.domain == node.id.clone() || edge.range == node.id.clone() {
                continue 'node_iter;
            }
        }
        isolated_nodes.push(node.id.clone());
    }
    return isolated_nodes;
}
// !SECTION