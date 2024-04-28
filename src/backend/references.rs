use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;

use super::Backend;

pub async fn references(backend: &Backend, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
    let file_path = params.text_document_position.text_document.uri.to_file_path().unwrap();
    let file_name = file_path.to_str().unwrap();
    if let Some(parser) = backend.parsers.get(file_name) {

        let node = parser.get_ident_node_at(params.text_document_position.position.line as usize, params.text_document_position.position.character as usize);

        if node.is_some() {

            eprintln!("got node: {:?}", node.unwrap().path);

            let node_path = &node.unwrap().path;
            let mut refs = Vec::new();

            for parser in backend.parsers.iter() {
                // eprintln!("parser: {:?}", parser.file);
                let mut nodes = Vec::new();
                nodes.push(parser.refs.get(node_path));
                if node_path.starts_with("/MICROSAR") {
                    nodes.push(parser.refs.get(&node_path.replace("/MICROSAR", "/AUTOSAR/EcucDefs")));
                }
                for ref_nodes in nodes {
                    if ref_nodes.is_some() {
                        for ref_node in ref_nodes.unwrap() { 
                            let location = Location {
                                uri: Url::from_file_path(&ref_node.file).unwrap(),
                                range: Range {
                                    start: Position::new(ref_node.start.row-1, ref_node.start.col-1),
                                    end: Position::new(ref_node.end.row-1, ref_node.end.col-1),
                                },
                            };
                            refs.push(location);
                        }
                    }
                }
            };

            Ok(Some(refs))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}