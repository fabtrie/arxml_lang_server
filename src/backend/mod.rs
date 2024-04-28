use std::path::PathBuf;

use anyhow::Error;
use dashmap::DashMap;
use regex::RegexSet;
use tokio::time::Instant;
use tower_lsp::{lsp_types::*, Client};
use rayon::prelude::*;

use crate::xml_parser::XmlParser;

mod init;
use init::init;
mod document_symbol;
use document_symbol::document_symbol;
mod hover;
use hover::hover;
mod symbol;
use symbol::symbol;
mod references;
use references::references;
mod goto_definition;
use goto_definition::goto_definition;
mod language_server;

pub struct Backend {
    client: Client,
    parsers: DashMap<String, XmlParser>,
    ws_folder: DashMap<String, WorkspaceFolder>,
    regex_dash: DashMap<String, RegexSet>,
    bool_dash: DashMap<String, bool>,
    parsing_pending_dash: DashMap<PathBuf, bool>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client: client,
            parsers: DashMap::new(),
            ws_folder: DashMap::new(),
            regex_dash: DashMap::new(),
            bool_dash: DashMap::new(),
            parsing_pending_dash: DashMap::new()
        }
    }


    fn parse_ws(&self) -> Vec<String> {

        let errors = DashMap::new();
        errors.insert("errors", Vec::new());

        let files = self.parsing_pending_dash.iter().map(|file| file.key().clone()).collect::<Vec<PathBuf>>();

        files.par_iter().for_each(|file| {
            if let Err(result) = self.create_parser_sync(file, true) {
                errors.get_mut("errors").unwrap().push(result.to_string());
            }
        });

        let x = errors.get("errors").unwrap().to_vec(); x
    }

    fn is_ws_file(&self, file: &PathBuf) -> bool {
        let regexs = self.regex_dash.get("ignorePattern").unwrap();
        for ws_older in self.ws_folder.iter() {
            if file.starts_with(ws_older.uri.to_file_path().unwrap()) && !regexs.is_match(file.to_str().unwrap().replace("\\", "/").as_str()) {
                return true;
            }
        }
        return false;
    }

    fn create_parser_sync(&self, file: &PathBuf, ignore_non_ws_files:bool) -> core::result::Result<(), Error> {
        let is_ws_file = self.is_ws_file(file);
        if ignore_non_ws_files && !is_ws_file {
            return Ok(());
        }
        let file_name = file.to_str().unwrap();
        self.parsing_pending_dash.insert(file.clone(), true);

        let mut parser = XmlParser::new(file_name, is_ws_file);

        let now = Instant::now();
        let result = parser.parse();
        match result {
            Ok(()) => {
                self.parsers.insert(file_name.to_string(), parser);
                eprintln!("parsing {} took: {:?}", file_name, now.elapsed());
            },
            Err(_) => {
                self.parsers.remove(file_name);
            }
        }
        self.parsing_pending_dash.remove(file);

        result
    }

    async fn create_parser(&self, file: &PathBuf)  -> core::result::Result<(), Error> {
        let result = self.create_parser_sync(file, false);
        if result.is_err() {
            self.client.log_message(MessageType::ERROR, format!("could not parse file: {:?}", result.as_ref().err().unwrap())).await;
        }
        result
    }
}