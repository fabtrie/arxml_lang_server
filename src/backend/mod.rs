use std::{collections::HashMap, path::PathBuf};

use anyhow::Error;
use dashmap::DashMap;
use regex::RegexSet;
use tokio::time::Instant;
use tower_lsp::{lsp_types::*, Client};
use rayon::prelude::*;
use glob::glob;

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

struct ClientConfig {
    ignore_regex_set: RegexSet,

}

pub struct Backend {
    client: Client,
    parsers: HashMap<String, XmlParser>,
    ws_folder: Vec<WorkspaceFolder>,
    config: Option<ClientConfig>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client: client,
            parsers: HashMap::new(),
            ws_folder: Vec::new(),
            config: None,

        }
    }

    fn parse_ws(&mut self) -> Vec<String> {

        let errors = DashMap::new();
        errors.insert("errors", Vec::new());

        let mut files: Vec<Result<PathBuf, glob::GlobError>> = Vec::new();

        self.ws_folder.iter().for_each(|folder| {
            let folder_path = folder.uri.to_file_path().unwrap();
            let folder_name = folder_path.to_str().unwrap();
            eprintln!("got workspace folder: {:?}", folder_path);
            let g = glob(&format!("{}/**/*.arxml", folder_name)).expect("Failed to read glob pattern");
            files.extend(g);
        });

        // create a vec of type Vec<PathBuf> containing files where the result is Ok and where is_ws_file returns true
        let files: Vec<PathBuf> = files
            .into_iter()
            .filter_map(|file| file.ok())
            .filter(|file| self.is_ws_file(file))
            .collect();
    
        self.parsers.par_extend(files.par_iter().filter_map(|file: &PathBuf| {
            if let Ok(parser) = Backend::create_parser_sync(file, true) {
                Some((file.to_str().unwrap().to_string(), parser))
            } else {
                None
            }
        }));

        let x = errors.get("errors").unwrap().to_vec(); x
    }

    fn is_ws_file(&self, file: &PathBuf) -> bool {
        let regexs = &self.config.as_ref().expect("Accessed config too early").ignore_regex_set;
        for ws_folder in self.ws_folder.iter() {
            if file.starts_with(ws_folder.uri.to_file_path().unwrap()) && !regexs.is_match(file.to_str().unwrap().replace("\\", "/").as_str()) {
                return true;
            }
        }
        return false;
    }

    fn create_parser_sync(file: &PathBuf, is_ws_file:bool) -> core::result::Result<XmlParser, Error> {
        
        let file_name = file.to_str().unwrap();

        let mut parser = XmlParser::new(file_name, is_ws_file);

        let now = Instant::now();
        let result = parser.parse();
        match result {
            Ok(()) => {
                eprintln!("parsing {} took: {:?}", file_name, now.elapsed());
                Ok(parser)
            },
            Err(e) => {
                Err(e)
            }
        }
    }

    async fn create_parser(&mut self, file: &PathBuf)  -> core::result::Result<(), Error> {
        let result = Backend::create_parser_sync(file, self.is_ws_file(file));
        let file_name = file.to_str().unwrap();
        match result {
            Ok(parser) => {
                self.parsers.insert(file_name.to_string(), parser);
                Ok(())
            },
            Err(e) => {
                self.parsers.remove(file_name);
                self.client.log_message(MessageType::ERROR, format!("could not parse file: {:?}", e)).await;
                Err(e)
            }
        }
    }
}