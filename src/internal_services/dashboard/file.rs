use rust_embed::RustEmbed;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::acceptors::traits::Sender;

#[derive(RustEmbed)]
#[folder = "src/internal_services/dashboard/website/public/"]
struct WebsiteFolder;

#[tracing::instrument(skip(request, sender))]
pub async fn handle_file(request: &Request<'_>, sender: &mut dyn Sender) -> Result<(), ()> {
    let raw_path = request.path().trim_start_matches('/');
    let raw_path = if raw_path.ends_with('/') || raw_path.is_empty() {
        format!("{}index.html", raw_path)
    } else {
        raw_path.to_owned()
    };

    let (path, content_type) = match raw_path.rsplit_once('.') {
        Some((_, ending)) => {
            let c_type = match ending {
                "html" => "text/html",
                "js" => "text/javascript",
                "css" => "text/css",
                _ => "text",
            };

            (raw_path.to_owned(), c_type)
        }
        None => ("index.html".to_owned(), "text/html"),
    };

    let file = match WebsiteFolder::get(&path) {
        Some(content) => content,
        None => {
            tracing::error!("Could not load File: {:?}", path);
            return Err(());
        }
    };

    let mut headers = Headers::new();
    headers.append("Content-Type", content_type);
    let content = match file {
        std::borrow::Cow::Borrowed(val) => {
            headers.append("Content-Length", val.len());
            val.to_vec()
        }
        std::borrow::Cow::Owned(val) => {
            headers.append("Content-Length", val.len());
            val.to_vec()
        }
    };
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    sender.send_response(&response).await;

    Ok(())
}
