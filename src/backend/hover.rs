
use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;

use super::Backend;

pub async fn hover(backend: &Backend, params: HoverParams) -> Result<Option<Hover>> {
    let file_path = params.text_document_position_params.text_document.uri.to_file_path().unwrap();
    let file_name = file_path.to_str().unwrap();
    let mut hover = None;
    if let Some(parser) = backend.parsers.get(file_name) {

        let node = parser.get_ident_node_at(params.text_document_position_params.position.line as usize, params.text_document_position_params.position.character as usize);
        if node.is_some() {
            let node = node.unwrap();

            eprintln!("got node: {:?}", node.node.file);

            let mut refs = "".to_owned();
            for (_, parser) in backend.parsers.iter() {
                // eprintln!("parser: {:?}", parser.file);
                let mut nodes = Vec::new();
                nodes.push(parser.refs.get(&node.path));
                if node.path.starts_with("/MICROSAR") {
                    nodes.push(parser.refs.get(&node.path.replace("/MICROSAR", "/AUTOSAR/EcucDefs")));
                }
                for ref_node in nodes {
                    if ref_node.is_some() {
                        // eprintln!("ref_node: {:?}", ref_node.unwrap().len());
                        refs.push_str(&ref_node.unwrap().iter().map(|ref_node| {
                            format!("[{}](file:///{}#{})\n", ref_node.path, ref_node.file.replace("\\", "/"), ref_node.start.row)
                        }).collect::<Vec<String>>().join(""));
                    }
                }
            }
            hover = Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**PATH:** [{}](file:///{}#{})\n### References\n{}", node.path, node.node.file.replace("\\", "/"), node.node.start.row, refs),
                }),
                range: None,
            });
        }
    }

    Ok(hover)
}