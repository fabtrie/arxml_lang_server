use tower_lsp::lsp_types::*;
use tower_lsp::jsonrpc::Result;

use super::Backend;

pub fn init(backend: &mut Backend, params: InitializeParams) -> Result<InitializeResult> {
    if let Some(ws_folders) = params.workspace_folders {
        backend.ws_folder = ws_folders;
    }
    
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