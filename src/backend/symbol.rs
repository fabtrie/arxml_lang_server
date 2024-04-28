use rayon::prelude::*;
use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;

use super::Backend;

pub async fn symbol(backend: &Backend, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
    let mut symbols: Vec<SymbolInformation> = Vec::new();

    if !params.query.is_empty() {

        eprintln!("query: {:?}", params.query);

        let mut vec_id_nodes: Vec<Vec<SymbolInformation>> = Vec::new();

        vec_id_nodes.par_extend(backend.parsers.par_iter().map(| parser | {
            let mut symbols = Vec::new();

            for node in parser.ident_nodes.values() {
                if node.short_name.to_lowercase().starts_with(&params.query.to_lowercase()) {
                    
                    #[allow(deprecated)]
                    let symbol = SymbolInformation {
                        name: node.short_name.clone(),
                        kind: SymbolKind::OBJECT,
                        tags: None,
                        location: Location {
                            uri: Url::from_file_path(&node.node.file).unwrap(),
                            range: Range {
                                start: Position::new(node.node.start.row-1, node.node.start.col-1),
                                end: Position::new(node.node.end.row-1, node.node.end.col-1),
                            },
                        },
                        container_name: Some(node.path.clone()),
                        deprecated: None,
                    };
                    symbols.push(symbol);
                }
            }
            symbols
        }));

        symbols.extend(vec_id_nodes.into_iter().flatten());

        eprintln!("symbols: {:?}", symbols.len());

        if symbols.len() > 10000 {
            symbols.drain(10000..symbols.len());
        }
    }
    Ok(Some(symbols))
}