use std::collections::{BTreeMap, HashMap};

use anyhow::Error;
use roxmltree::{Document, Node, TextPos};
use tower_lsp::lsp_types::SymbolKind;

pub struct XmlParserNode{
    pub file: String,
    pub start: TextPos,
    pub end: TextPos,
    pub range: std::ops::Range<usize>,
    pub tag_name: String,
    pub def_ref: Option<String>,
}

pub struct IdentNode {
    pub short_name: String,
    pub node: XmlParserNode,
    pub short_name_start: TextPos,
    pub short_name_end: TextPos,
    pub short_name_range: std::ops::Range<usize>,
    pub path: String,
    pub values: Vec<ValueNode>,
}

pub struct ValueNode {
    pub short_name: String,
    pub node: XmlParserNode,
    pub path: String,
    pub value: String,
    pub typ: SymbolKind
}

pub struct RefNode {
    pub file: String,
    pub start: TextPos,
    pub end: TextPos,
    pub range: std::ops::Range<usize>,
    pub tag_name: String,
    pub text_start: TextPos,
    pub text_end: TextPos,
    pub text: String,
    pub text_range: std::ops::Range<usize>,
    pub path: String,
}

pub struct XmlParser{
    pub ident_nodes: BTreeMap<String, IdentNode>,
    pub value_nodes: Vec<ValueNode>,
    pub refs: HashMap<String, Vec<RefNode>>,
    line_offsets: Vec<usize>,
    pub file: String,
    last_ident_node: Option<String>,
    pub is_ws_file: bool,
    pub vendor_mapping: Option<(String, String)>,
}

impl XmlParser {
    pub fn new(file_name: &str, is_ws_file: bool) -> Self {
        let xml_parser = XmlParser {ident_nodes: BTreeMap::new(), refs: HashMap::new(), line_offsets: Vec::new(), file: file_name.to_string(), value_nodes: Vec::new(), last_ident_node: None, is_ws_file: is_ws_file, vendor_mapping: None};
        xml_parser
    }

    pub fn parse_file(&mut self) -> Result<(), Error> {
        let content = std::fs::read_to_string(self.file.to_string())?;
        self.parse(&content)
    }

    pub fn parse(&mut self, content: &str) -> Result<(), Error> {
        // eprintln!("reading file: {}", self.file);
        // let now = Instant::now();
        // let content = std::fs::read_to_string(self.file.to_string())?;
        self.line_offsets = get_line_offsets(&content);
        // let elapsed = now.elapsed();

        // let now = Instant::now();
        // eprintln!("parsing file");
        let doc = Some(Document::parse(&content)?);
        // let elapsed2 = now.elapsed();

        // let now = Instant::now();
        // eprintln!("traversing file");
        self.traverse_xml("".to_string(), None, &doc.unwrap());
        // let elapsed3 = now.elapsed();

        // eprintln!("Read file: {:?}, parse: {:?}, traverse: {:?}", elapsed, elapsed2, elapsed3);


        Ok(())
    }

    fn traverse_xml<'a>(&mut self, path: String, doc: Option<&Node<'a, 'a>>, doc2: &Document<'a>) {
        let binding = doc2.root();
        let doc = match doc {
            Some(doc) => doc,
            None => &binding,
        };
        
        for child in doc.children() {
            let mut new_path = path.clone();
            let tag_name = child.tag_name().name();

            if let Some(short_name) = get_short_name_node(child) {
                if let Some(short_name_text) = short_name.text() {
                    new_path.push_str(&format!("/{}", short_name_text));
                    let (start, end) = self.get_text_pos(child.range());
                    let(short_name_start, short_name_end) = self.get_text_pos(short_name.first_child().unwrap().range());

                    let def_ref = if tag_name == "ECUC-CONTAINER-VALUE" {
                        if let Some(def_ref) = child.children().find(|child1| child1.tag_name().name() == "DEFINITION-REF") {
                            if let Some(def_ref_text) = def_ref.text() {
                                Some(def_ref_text.to_string())
                            } else {
                                let start_pos = self.get_text_pos(def_ref.range()).0;
                                eprint!("ERROR: No text found for node: {}:{}:{}\n", self.file.to_string(), start_pos.row, start_pos.col);
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let node = IdentNode {
                        short_name: short_name_text.to_string(),
                        node: XmlParserNode {
                            // doc: self.doc,
                            file: self.file.to_string(),
                            start: start,
                            end: end,
                            range: child.range(),
                            tag_name: tag_name.to_string(),
                            def_ref: def_ref,

                        },
                        short_name_start: short_name_start,
                        short_name_end: short_name_end,
                        short_name_range: short_name.first_child().unwrap().range(),
                        path: new_path.clone(),
                        values: Vec::new(),
                    };
                    self.ident_nodes.insert(new_path.to_owned(), node);
                    self.last_ident_node = Some(new_path.clone());
                } else {
                    let start_pos = self.get_text_pos(short_name.range()).0;
                    eprint!("ERROR: No text found for node: {}:{}:{}\n", self.file.to_string(), start_pos.row, start_pos.col);
                }
                
            } else if child.has_attribute("DEST") {
                if let Some(ref_text) = child.text() {
                    let (start, end) = self.get_text_pos(child.range());
                    let text_node = child.first_child().unwrap();
                    let (text_start, text_end) = self.get_text_pos(text_node.range());
                    let node = RefNode {
                        // doc: self.doc,
                        file: self.file.to_string(),
                        start: start,
                        end: end,
                        range: child.range(),
                        tag_name: tag_name.to_string(),
                        text: ref_text.to_string(),
                        text_start: text_start,
                        text_end: text_end,
                        text_range: text_node.range(),
                        path: new_path.clone(),
                    };
                    if let Some(ref_vec) = self.refs.get_mut(ref_text) {
                        ref_vec.push(node);
                    } else {
                        self.refs.insert(ref_text.to_string(), vec![node]);
                    }
                    if tag_name == "REFINED-MODULE-DEF-REF" {
                        let module = ref_text;
                        self.vendor_mapping = Some((new_path.to_owned(), module.to_string()));
                        eprintln!("Vendor Mapping: {} -> {}", new_path, module);
                    }
                } else {
                    let start_pos = self.get_text_pos(child.range()).0;
                    eprint!("ERROR: No text found for ref node: {}:{}:{}\n", self.file.to_string(), start_pos.row, start_pos.col);
                }
            } else if tag_name == "ECUC-CONTAINER-VALUE" || tag_name == "ECUC-REFERENCE-VALUE" || tag_name == "ECUC-NUMERICAL-PARAM-VALUE" || tag_name == "ECUC-TEXTUAL-PARAM-VALUE" {
                let def_ref_node = child.children().find(|child| child.tag_name().name() == "DEFINITION-REF");

                if def_ref_node.is_some() {
                    let def_ref_node = def_ref_node.unwrap();
                    if let Some(def_ref_text) = def_ref_node.text() {
                        let def_ref = def_ref_text;
                        let name = def_ref.split('/').last().unwrap();
                        let (start, end) = self.get_text_pos(child.range());
                        let path = format!("{}/{}", new_path, name);
                        let dest_type = def_ref_node.attribute("DEST");

                        let typ = match tag_name {
                            "ECUC-TEXTUAL-PARAM-VALUE" => {
                                match dest_type {
                                    Some("ECUC-ENUMERATION-PARAM-DEF") => SymbolKind::ENUM,
                                    Some("ECUC-FUNCTION-NAME-DEF") => SymbolKind::FUNCTION,
                                    _ => SymbolKind::STRING,
                                }
                            },
                            "ECUC-NUMERICAL-PARAM-VALUE" => {
                                match dest_type {
                                    Some("ECUC-BOOLEAN-PARAM-DEF") => SymbolKind::BOOLEAN,
                                    _ => SymbolKind::NUMBER,
                                }
                            },
                            _ => SymbolKind::VARIABLE,
                        };

                        let value = if tag_name == "ECUC-REFERENCE-VALUE" {
                            child.children().find(|child| child.tag_name().name() == "VALUE-REF")
                        } else {
                            child.children().find(|child| child.tag_name().name() == "VALUE")
                        };
                        let value = match value {
                            Some(value) => {
                                match value.text() {
                                    Some(value) => value,
                                    None => "",
                                }
                            },
                            None => "",
                        };

                        let value = ValueNode {
                            short_name: name.to_string(),
                            node: XmlParserNode {
                                // doc: self.doc,
                                file: self.file.to_string(),
                                start: start,
                                end: end,
                                range: child.range(),
                                tag_name: tag_name.to_string(),
                                def_ref: Some(def_ref.to_string()),
                            },
                            path: path.clone(),
                            value: value.to_string(),
                            typ: typ
                        };
                        self.ident_nodes.get_mut(self.last_ident_node.as_ref().unwrap()).unwrap().values.push(value);
                    } else {
                        let start_pos = self.get_text_pos(def_ref_node.range()).0;
                        eprint!("ERROR: No text found for node: {}:{}:{}\n", self.file.to_string(), start_pos.row, start_pos.col);
                    }
                }
            }
            self.traverse_xml(new_path, Some(&child), doc2);
        }
    }


    fn get_text_pos(&mut self, range: std::ops::Range<usize>) -> (TextPos, TextPos) {
        let start = range.start;
        let end = range.end;
        
        let start_line = self.line_offsets.partition_point(|&x| x <= start);
        let end_line = self.line_offsets.partition_point(|&x| x <= end);
        let start_row = start - self.line_offsets[start_line-1]+1;
        let end_row = end - self.line_offsets[end_line-1]+1;

        (TextPos::new(start_line as u32, start_row as u32), TextPos::new(end_line as u32, end_row as u32))
    }

    pub fn get_ident_node_at(&self, line: usize, position: usize) -> Option<&IdentNode> {
        let offset = self.line_offsets.get(line)? + position;

        for node in self.ident_nodes.values().rev() {
            let start = node.node.range.start;
            let end = node.node.range.end;
            if start <= offset && offset <= end {
                return Some(node);
            }
        }
        None
    }

    pub fn get_ref_text_at(&self, line: usize, position: usize) -> Option<(String, usize)> {
        let offset = self.line_offsets.get(line)? + position;

        for node in self.refs.values().flatten() {
            let start = node.text_range.start;
            let end = node.text_range.end;
            if start <= offset && offset <= end {
                let curser_pos = offset - start;
                let text = node.text.as_str();
                let mut pos: usize = 0;
                let mut text_part = "".to_owned();

                text.split('/').for_each(|s| {
                    if s.len() != 0 {
                        if pos <= curser_pos {
                            text_part.push_str("/");
                            text_part.push_str(s);
                        }
                        pos = pos + s.len() + 1;
                    }
                });
                return Some((text_part, curser_pos));
            }
        }
        None
    }
}

fn get_line_offsets(text: &str) -> Vec<usize> {
    let mut line_offsets = vec![0];
    let mut offset = 0;
    for c in text.chars() {
        offset += c.len_utf8();
        if c == '\n' {
            line_offsets.push(offset);
        }
    }
    line_offsets
}

fn get_short_name_node<'a>(node: Node<'a, 'a>) -> Option<Node<'a, 'a>> {
    node.children().find(|child| child.tag_name().name() == "SHORT-NAME")
}
