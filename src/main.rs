use std::env;
// use bugsalot::debugger;
use tokio::net::TcpStream;
use tower_lsp::{LspService, Server};
mod xml_parser;
mod backend;

use backend::Backend;

#[tokio::main]
async fn main() {
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    let args: Vec<String> = env::args().collect();

    // debugger::wait_until_attached(None).expect("state() not implemented on this platform");

    if args.len() > 1 {
        // the first argument is the name of this executable
        let address = args.get(1).expect("first argument must be the address");
        let port = args.get(2).expect("second argument must be the port");
        let connect_str = format!("{}:{}", address, port);

        let stream = TcpStream::connect(connect_str).await.unwrap();

        let (read, write) = tokio::io::split(stream);
        #[cfg(feature = "runtime-agnostic")]
        let (read, write) = (read.compat(), write.compat_write());

        let (service, socket) = LspService::new(|client| Backend::new(client));
        Server::new(read, write, socket).serve(service).await;
    } else {
        let (read, write) = (tokio::io::stdin(), tokio::io::stdout());
        #[cfg(feature = "runtime-agnostic")]
        let (read, write) = (read.compat(), write.compat_write());

        let (service, socket) = LspService::new(|client| Backend::new(client));
        Server::new(read, write, socket).serve(service).await;
    }
}
