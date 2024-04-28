use glob::glob;
use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;

use super::Backend;

pub fn init(backend: &Backend, params: InitializeParams) -> Result<InitializeResult> {
    if let Some(ws_folders) = params.workspace_folders {
        for folder in ws_folders {
            let folder_path = folder.uri.to_file_path().unwrap();
            let folder_name = folder_path.to_str().unwrap();
            backend.ws_folder.insert(folder_name.to_string(), folder);
            eprintln!("got workspace folder: {:?}", folder_path);

            let glob_files = glob(&format!("{}/**/*.arxml", folder_name)).expect("Failed to read glob pattern");

            glob_files.for_each(|file| {
                if let Ok(file) = file {
                    backend.parsing_pending_dash.insert(file, true);
                }
            });
        }
    }

    eprintln!("files inserted");

    Ok(InitializeResult {
        server_info: None,
        capabilities: ServerCapabilities { 
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders:Some(WorkspaceFoldersServerCapabilities{
                    supported:Some(true),
                    change_notifications:Some(OneOf::Left(true)),}),
                    file_operations: None }),
            document_symbol_provider: Some(OneOf::Left(true)),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::NONE),
                will_save: None,
                will_save_wait_until: None,
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions{include_text: None})),
            })),
            ..Default::default()
        }
    })
}