
use std::collections::HashMap;

use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;

use super::Backend;

pub async fn document_symbol(backend: &Backend, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
    eprintln!("document symbols requested!");
    backend.client.log_message(MessageType::INFO, "document symbols requested!").await;
    let file_path = params.text_document.uri.to_file_path().unwrap();
    let file_name = file_path.to_str().unwrap();

    if let Some(parser) = backend.parsers.get(file_name) {

        let mut root_symbols = HashMap::new();

        parser.ident_nodes.values().for_each(|node| {
            let parent_path = node.path.rsplit_once('/').unwrap().0;
            let path_length = parent_path.split('/').count();

            let detail = if node.node.def_ref.is_some() {
                Some(node.node.def_ref.as_ref().unwrap().rsplit_once('/').unwrap().1.to_string())
            } else {
                Some(node.node.tag_name.to_string())
            };

            let typ = match node.node.tag_name.as_str() {
                "ECUC-CONTAINER-VALUE" => SymbolKind::STRUCT,
                "ECUC-PARAM-CONF-CONTAINER-DEF" => SymbolKind::STRUCT,
                "ECUC-BOOLEAN-PARAM-DEF" => SymbolKind::BOOLEAN,
                "ECUC-ENUMERATION-PARAM-DEF" => SymbolKind::ENUM,
                "ECUC-ENUMERATION-LITERAL-DEF" => SymbolKind::ENUM_MEMBER,
                "ECUC-REFERENCE-DEF" => SymbolKind::VARIABLE,
                "ECUC-SYMBOLIC-NAME-REFERENCE-DEF" => SymbolKind::VARIABLE,
                "ECUC-INTEGER-PARAM-DEF" => SymbolKind::NUMBER,
                "ECUC-STRING-PARAM-DEF" => SymbolKind::STRING,
                "ECUC-FUNCTION-NAME-DEF" => SymbolKind::FUNCTION,
                _ => SymbolKind::OBJECT,
            };

            #[allow(deprecated)]
            let mut symbol = DocumentSymbol {
                name: node.short_name.clone(),
                detail: detail,
                kind: typ,
                deprecated: None,
                range: Range {
                    start: Position::new(node.node.start.row-1, node.node.start.col-1),
                    end: Position::new(node.node.end.row-1, node.node.end.col-1),
                },
                selection_range: Range {
                    start: Position::new(node.short_name_start.row-1, node.short_name_start.col-1),
                    end: Position::new(node.short_name_end.row-1, node.short_name_end.col-1),
                },
                children: Some(Vec::new()),
                tags: None,
            };

            #[allow(deprecated)]
            node.values.iter().for_each(|value| {
                let detail = Some("= ".to_string() + &value.value);
                let value_symbol = DocumentSymbol {
                    name: value.short_name.clone(),
                    detail: detail,
                    kind: value.typ,
                    deprecated: None,
                    range: Range {
                        start: Position::new(value.node.start.row-1, value.node.start.col-1),
                        end: Position::new(value.node.end.row-1, value.node.end.col-1),
                    },
                    selection_range: Range {
                        start: Position::new(value.node.start.row-1, value.node.start.col-1),
                        end: Position::new(value.node.end.row-1, value.node.end.col-1),
                    },
                    children: None,
                    tags: None,
                };
                symbol.children.as_mut().unwrap().push(value_symbol);
            });

            if node.path == "/MICROSAR" {
                eprintln!("length: {}", path_length);
            }

            if path_length == 1 {
                root_symbols.insert(node.path.to_string(), symbol);
            } else {
                let split = parent_path.split('/').collect::<Vec<&str>>();
                // take the last element in the root_symbols and go down its children by always taking the last one as often as split.len()
                // insert the symbol into the last element
                let mut parent = root_symbols.iter_mut().last().unwrap().1;
                for _ in 2..split.len() {
                    parent = parent.children.as_mut().unwrap().iter_mut().last().unwrap();
                }
                parent.children.as_mut().unwrap().push(symbol);
                
            }
        });
        backend.client
            .log_message(MessageType::INFO, "document symbols request done!")
            .await;

        Ok(Some(DocumentSymbolResponse::Nested(root_symbols.into_values().collect())))
    } else {
        Ok(None)
    }
}