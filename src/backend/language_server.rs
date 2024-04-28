use regex::RegexSet;
use tokio::time::Instant;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;
use tower_lsp::jsonrpc::Result;

use super::Backend;

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        super::init(self, params)
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "initialized!").await;
        
        let result = self.client.configuration(vec![ConfigurationItem{scope_uri: None, section: Some("arxmlLanguageServer.ignorePattern".to_string()) }]).await;

        let regexs = RegexSet::new(result.unwrap()[0].as_array().unwrap().iter().map(|x| x.as_str().unwrap())).unwrap();

        self.regex_dash.insert("ignorePattern".to_string(), regexs);

        self.bool_dash.insert("init_done".to_string(), true);

        let now = Instant::now();
        self.parse_ws();
        let elapsed = now.elapsed();
        eprintln!("parsing took: {:?}", elapsed);
    }

    async fn shutdown(&self) -> Result<()> {
        // self.client
        //     .log_message(MessageType::INFO, "shutting down!")
        //     .await;
        eprintln!("shutting down!");
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        self.client.log_message(MessageType::INFO, "watched files have changed!").await;
        params.changes.iter().for_each(|change| {
            if change.typ == FileChangeType::DELETED {
                self.parsers.remove(change.uri.to_file_path().unwrap().to_str().unwrap());
            }
        });
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "file opened!").await;

        while self.bool_dash.get("init_done").is_none() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let file_path = params.text_document.uri.to_file_path().unwrap();
        let file_name = file_path.to_str().unwrap();

        if !self.parsers.contains_key(file_name) {
            let _ = self.create_parser(&file_path).await;
        }
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let file_path = params.text_document.uri.to_file_path().unwrap();

        let _ = self.create_parser(&file_path).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;

        let file_path = params.text_document.uri.to_file_path().unwrap();
        let file_name = file_path.to_str().unwrap();

        if let Some(parser) = self.parsers.get(file_name) {
            if ! parser.is_ws_file {
                // it is not allowed to remove an element if some reference into the map is used
                drop(parser);
                self.parsers.remove(file_name);
                self.client.log_message(MessageType::INFO, "removing parser!").await;
            }
        }
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        super::document_symbol(self, params).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        super::hover(self, params).await
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
        super::symbol(self, params).await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        super::references(self, params).await
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        super::goto_definition(self, params).await
    }
}
