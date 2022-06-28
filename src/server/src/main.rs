#[macro_use]
extern crate log;

use clap::App;
use lsp_types::{
    request::{GotoDefinition, HoverRequest},
    CodeDescription, Diagnostic, DiagnosticSeverity, GotoDefinitionResponse, Hover, HoverContents,
    HoverProviderCapability, InitializeParams, Location, MarkedString, NumberOrString, OneOf,
    Position, PublishDiagnosticsParams, Range, ServerCapabilities, Url,
};

// use treecore_ls::stdio_server;
use std::error::Error;

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let _matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .get_matches();
    env_logger::init();
    info!("Starting language server");

    // NOTE: we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("starting example main loop");

    let notif_tmp = PublishDiagnosticsParams {
        uri: Url::parse("file:///root/Desktop/ide/tree-core-backend/client/tests/trace.rvs")?,
        diagnostics: vec![Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 1,
                },
                end: Position {
                    line: 1,
                    character: 5,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::Number(1234234)),
            code_description: Some(CodeDescription {
                href: Url::parse(
                    "file:///root/Desktop/ide/tree-core-backend/client/tests/trace.rvs",
                )?,
            }),
            source: Some("demo".to_string()),
            message: "this is the first diagnostic".to_string(),
            related_information: None,
            tags: None,
            data: None,
        }],
        version: None,
    };

    let notification = lsp_server::Notification {
        method: "textDocument/publishDiagnostics".into(),
        params: serde_json::to_value(notif_tmp).unwrap(),
    };

    connection
        .sender
        .send(Message::Notification(notification))
        .unwrap();

    for msg in &connection.receiver {
        eprintln!("got msg: {:?}", msg);
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {:?}", req);
                // HACK: don't use 'clone' method
                match cast::<HoverRequest>(req.clone()) {
                    Ok((id, params)) => {
                        eprintln!("got hover request #{}: {:?}", id, params);
                        let result = Some(Hover {
                            contents: HoverContents::Scalar(MarkedString::String(
                                "I am maksyuki!!!".to_string(),
                            )),
                            range: None,
                        });
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };

                match cast::<GotoDefinition>(req) {
                    Ok((id, params)) => {
                        eprintln!("got gotoDefinition request #{}: {:?}", id, params);
                        let result = Some(GotoDefinitionResponse::Scalar(Location {
                            uri: Url::parse("https://example.net/a/c.png")?,
                            range: Range {
                                start: Position {
                                    line: 6,
                                    character: 6,
                                },
                                end: Position {
                                    line: 666666,
                                    character: 666666,
                                },
                            },
                        }));
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(notif) => {
                eprintln!("got notification: {:?}", notif);
            }
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
