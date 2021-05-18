use std::sync::Arc;

use serde::Serialize;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::{
    acceptors::traits::{Acceptor, Sender},
    configurator::{MiddlewareList, ServiceList},
    rules::{Middleware, ReadManager, Rule, Service},
};

#[derive(Debug, Serialize)]
struct AllAcceptorsResponse {
    acceptors: Vec<String>,
}

pub async fn handle_acceptors(
    request: &Request<'_>,
    sender: &mut dyn Sender,
    acceptors: &[Box<dyn Acceptor + Send + Sync + 'static>],
) -> Result<(), ()> {
    let mut final_acceptors = Vec::with_capacity(acceptors.len());
    for tmp in acceptors.iter() {
        final_acceptors.push(tmp.get_name());
    }

    let raw_content = AllAcceptorsResponse {
        acceptors: final_acceptors,
    };
    let content = serde_json::to_vec(&raw_content).unwrap();

    let mut headers = Headers::new();
    headers.append("Content-Length", content.len());
    headers.append("Content-Type", "application/json");
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    let (response_head, response_body) = response.serialize();
    let head_length = response_head.len();
    sender.send(response_head, head_length).await;
    let body_length = response_body.len();
    sender.send(response_body.to_vec(), body_length).await;

    Ok(())
}

#[derive(Debug, Serialize)]
struct AllConfiguratorsResponse {}

pub async fn handle_configurators(
    request: &Request<'_>,
    sender: &mut dyn Sender,
) -> Result<(), ()> {
    // TODO

    let raw_content = AllConfiguratorsResponse {};
    let content = serde_json::to_vec(&raw_content).unwrap();

    let mut headers = Headers::new();
    headers.append("Content-Length", content.len());
    headers.append("Content-Type", "application/json");
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    let (response_head, response_body) = response.serialize();
    let head_length = response_head.len();
    sender.send(response_head, head_length).await;
    let body_length = response_body.len();
    sender.send(response_body.to_vec(), body_length).await;

    Ok(())
}

#[derive(Debug, Serialize)]
struct AllRulesResponse {
    rules: Vec<Rule>,
}

pub async fn handle_rules(
    request: &Request<'_>,
    sender: &mut dyn Sender,
    rule_manager: &ReadManager,
) -> Result<(), ()> {
    let all_rules = rule_manager.get_all_rules().unwrap();

    let mut final_rules = Vec::with_capacity(all_rules.len());
    for tmp in all_rules {
        final_rules.push(Rule::clone(tmp.as_ref()));
    }

    let raw_content = AllRulesResponse { rules: final_rules };
    let content = serde_json::to_vec(&raw_content).unwrap();

    let mut headers = Headers::new();
    headers.append("Content-Length", content.len());
    headers.append("Content-Type", "application/json");
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    let (response_head, response_body) = response.serialize();
    let head_length = response_head.len();
    sender.send(response_head, head_length).await;
    let body_length = response_body.len();
    sender.send(response_body.to_vec(), body_length).await;

    Ok(())
}

#[derive(Debug, Serialize)]
struct AllServicesResponse {
    services: Vec<Service>,
}

pub async fn handle_services(
    request: &Request<'_>,
    sender: &mut dyn Sender,
    service_list: &ServiceList,
) -> Result<(), ()> {
    let all_services = service_list.get_all();

    let mut final_services = Vec::with_capacity(all_services.len());
    for tmp in all_services {
        final_services.push(Service::clone(tmp.as_ref()));
    }

    let raw_content = AllServicesResponse {
        services: final_services,
    };
    let content = serde_json::to_vec(&raw_content).unwrap();

    let mut headers = Headers::new();
    headers.append("Content-Length", content.len());
    headers.append("Content-Type", "application/json");
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    let (response_head, response_body) = response.serialize();
    let head_length = response_head.len();
    sender.send(response_head, head_length).await;
    let body_length = response_body.len();
    sender.send(response_body.to_vec(), body_length).await;

    Ok(())
}

#[derive(Debug, Serialize)]
struct AllMiddlewaresResponse {
    middlewares: Vec<Middleware>,
}

pub async fn handle_middlewares(
    request: &Request<'_>,
    sender: &mut dyn Sender,
    middleware_list: &MiddlewareList,
) -> Result<(), ()> {
    let all_middlewares = middleware_list.get_all();

    let mut final_middlewares = Vec::with_capacity(all_middlewares.len());
    for tmp in all_middlewares {
        final_middlewares.push(Middleware::clone(tmp.as_ref()));
    }

    let raw_content = AllMiddlewaresResponse {
        middlewares: final_middlewares,
    };
    let content = serde_json::to_vec(&raw_content).unwrap();

    let mut headers = Headers::new();
    headers.append("Content-Length", content.len());
    headers.append("Content-Type", "application/json");
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    let (response_head, response_body) = response.serialize();
    let head_length = response_head.len();
    sender.send(response_head, head_length).await;
    let body_length = response_body.len();
    sender.send(response_body.to_vec(), body_length).await;

    Ok(())
}
