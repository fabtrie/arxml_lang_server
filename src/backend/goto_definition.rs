use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;

use super::Backend;

pub async fn goto_definition(backend: &Backend, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    let file_path = params.text_document_position_params.text_document.uri.to_file_path().expect("Failed to convert URI to path");
    let file_name = file_path.to_str().expect("Failed to convert path to string");
    if let Some(parser) = backend.parsers.get(file_name) {

        if let Some(text) = parser.get_ref_text_at(params.text_document_position_params.position.line as usize, params.text_document_position_params.position.character as usize) {
            let path = text.0.as_str();
            let node_start = text.1 as u32;
            eprintln!("{:?}", path);

            let mut node_list = Vec::new();

            backend.parsers.iter().for_each(|parser| {
                let mut nodes = Vec::new();
                nodes.push(parser.ident_nodes.get(path));
                if let Some(vendor_mapping) = &parser.vendor_mapping {
                    if path.starts_with(vendor_mapping.1.as_str()) {
                        nodes.push(parser.ident_nodes.get(&path.replace(vendor_mapping.1.as_str(), vendor_mapping.0.as_str())));
                    }
                }

                for node in nodes {
                    if let Some(node) = node {
                        let target_range = Range {
                            start: Position::new(node.node.start.row-1, node.node.start.col-1),
                            end: Position::new(node.node.end.row-1, node.node.end.col-1)
                        };
                        let target_selection_range = Range {
                            start: Position::new(node.short_name_start.row-1, node.short_name_start.col-1),
                            end: Position::new(node.short_name_end.row-1, node.short_name_end.col-1)
                        };
                        let location = LocationLink {
                            origin_selection_range: Some(Range {
                                start: Position::new(params.text_document_position_params.position.line, params.text_document_position_params.position.character-node_start),
                                end: Position::new(params.text_document_position_params.position.line, params.text_document_position_params.position.character-node_start+path.len() as u32)
                            }),
                            target_uri: Url::from_file_path(&node.node.file).expect(format!("Failed to convert path to URI: {:?}", &node.node.file).as_str()),
                            target_range: target_range,
                            target_selection_range: target_selection_range,
                        };
                        node_list.push(location);
                    }
                }
            });

            return Ok(Some(GotoDefinitionResponse::Link(node_list)));
        }
    }
    return Ok(None);
}