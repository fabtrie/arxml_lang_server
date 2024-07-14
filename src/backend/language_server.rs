use regex::RegexSet;
use tokio::time::Instant;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;
use tower_lsp::jsonrpc::Result;

use crate::backend::ClientConfig;

use super::Backend;

#[tower_lsp::async_trait(?Send)]
impl LanguageServer for Backend {
    async fn initialize(&mut self, params: InitializeParams) -> Result<InitializeResult> {
        super::init(self, params)
    }

    async fn initialized(&mut self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "initialized!").await;
        
        let result = self.client.configuration(vec![ConfigurationItem{scope_uri: None, section: Some("arxmlLanguageServer.ignorePattern".to_string()) }]).await;

        let regexs = RegexSet::new(result.unwrap()[0].as_array().unwrap().iter().map(|x| x.as_str().unwrap())).unwrap();

        self.config = Some(ClientConfig {
            ignore_regex_set: regexs,
        });

        let now = Instant::now();
        self.parse_ws();
        let elapsed = now.elapsed();
        eprintln!("parsing took: {:?}", elapsed);
    }

    async fn shutdown(&mut self) -> Result<()> {
        // self.client
        //     .log_message(MessageType::INFO, "shutting down!")
        //     .await;
        eprintln!("shutting down!");
        Ok(())
    }

    async fn did_change_workspace_folders(&mut self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&mut self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&mut self, params: DidChangeWatchedFilesParams) {
        self.client.log_message(MessageType::INFO, "watched files have changed!").await;
        params.changes.iter().for_each(|change| {
            if change.typ == FileChangeType::DELETED {
                self.parsers.remove(change.uri.to_file_path().unwrap().to_str().unwrap());
            }
        });
    }

    async fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "file opened!").await;

        let file_path = params.text_document.uri.to_file_path().unwrap();
        let file_name = file_path.to_str().unwrap();

        if !self.parsers.contains_key(file_name) {
            let _ = self.create_parser(&file_path, None).await;
        }
    }

    async fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        
        let file_path = params.text_document.uri.to_file_path().unwrap();
        let file_name = file_path.to_str().unwrap();
        if self.parsers.contains_key(file_name) {
            self.parsers.remove(file_name);
        }

        // let mut f = File::create("foo.txt").unwrap();
        // f.write_all(params.content_changes.get(0).unwrap().text.as_bytes()).unwrap();

        let result = Backend::create_parser_sync(&file_path, self.is_ws_file(&file_path), Some(&params.content_changes.get(0).unwrap().text));
        match result {
            Ok(parser) => {
                self.parsers.insert(file_name.to_string(), parser);
            },
            Err(e) => {
                self.client.log_message(MessageType::ERROR, format!("could not parse file: {:?}", e)).await;
            }
        }
    }

    async fn did_save(&mut self, params: DidSaveTextDocumentParams) {
        let file_path = params.text_document.uri.to_file_path().unwrap();

        let _ = self.create_parser(&file_path, None).await;
    }

    async fn did_close(&mut self, params: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;

        let file_path = params.text_document.uri.to_file_path().unwrap();
        let file_name = file_path.to_str().unwrap();

        if let Some(parser) = self.parsers.get(file_name) {
            if !parser.is_ws_file {
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
