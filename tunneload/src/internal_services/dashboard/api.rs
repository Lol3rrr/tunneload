use serde::Serialize;
use stream_httparse::{Headers, Request, Response, StatusCode};

use crate::configurator::{MiddlewareList, PluginList, ServiceList};
use general_traits::Sender;
use plugins::Plugin;
use rules::{Middleware, ReadManager, Rule, Service};

use super::DashboardEntityList;

#[derive(Serialize)]
struct AllAcceptorsResponse<'a> {
    acceptors: &'a DashboardEntityList,
}

pub async fn handle_acceptors(
    _request: &Request<'_>,
    sender: &mut dyn Sender,
    acceptors: &DashboardEntityList,
) -> Result<(), ()> {
    let raw_content = AllAcceptorsResponse { acceptors };
    let content = serde_json::to_vec(&raw_content).unwrap();

    let mut headers = Headers::new();
    headers.append("Content-Length", content.len());
    headers.append("Content-Type", "application/json");
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    sender.send_response(&response).await;

    Ok(())
}

#[derive(Serialize)]
struct AllConfiguratorsResponse<'a> {
    configurators: &'a DashboardEntityList,
}

pub async fn handle_configurators(
    _request: &Request<'_>,
    sender: &mut dyn Sender,
    configurators: &DashboardEntityList,
) -> Result<(), ()> {
    let raw_content = AllConfiguratorsResponse { configurators };
    let content = serde_json::to_vec(&raw_content).unwrap();

    let mut headers = Headers::new();
    headers.append("Content-Length", content.len());
    headers.append("Content-Type", "application/json");
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    sender.send_response(&response).await;

    Ok(())
}

#[derive(Debug, Serialize)]
struct AllRulesResponse {
    rules: Vec<Rule>,
}

pub async fn handle_rules(
    _request: &Request<'_>,
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

    sender.send_response(&response).await;

    Ok(())
}

#[derive(Debug, Serialize)]
struct AllServicesResponse {
    services: Vec<Service>,
}

pub async fn handle_services(
    _request: &Request<'_>,
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

    sender.send_response(&response).await;

    Ok(())
}

#[derive(Debug, Serialize)]
struct AllMiddlewaresResponse {
    middlewares: Vec<Middleware>,
}

pub async fn handle_middlewares(
    _request: &Request<'_>,
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

    sender.send_response(&response).await;

    Ok(())
}

#[derive(Debug, Serialize)]
struct AllPluginsResponse {
    plugins: Vec<Plugin>,
}

pub async fn handle_plugins(
    _request: &Request<'_>,
    sender: &mut dyn Sender,
    plugin_list: &PluginList,
) -> Result<(), ()> {
    let all_plugins = plugin_list.get_all();

    let mut final_plugins = Vec::with_capacity(all_plugins.len());
    for tmp in all_plugins {
        final_plugins.push(Plugin::clone(tmp.as_ref()));
    }

    let raw_content = AllPluginsResponse {
        plugins: final_plugins,
    };
    let content = serde_json::to_vec(&raw_content).unwrap();

    let mut headers = Headers::new();
    headers.append("Content-Length", content.len());
    headers.append("Content-Type", "application/json");
    let response = Response::new("HTTP/1.1", StatusCode::OK, headers, content);

    sender.send_response(&response).await;

    Ok(())
}
