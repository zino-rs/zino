use crate::application::Application;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use metrics_exporter_tcp::TcpBuilder;
use std::{net::IpAddr, time::Duration};

pub(super) fn init<APP: Application + ?Sized>() {
    if let Some(metrics) = APP::config().get("metrics").and_then(|t| t.as_table()) {
        let exporter = metrics
            .get("exporter")
            .and_then(|t| t.as_str())
            .unwrap_or_default();
        if exporter == "prometheus" {
            let mut builder = match metrics.get("push-gateway").and_then(|t| t.as_str()) {
                Some(endpoint) => {
                    let interval = metrics
                        .get("interval")
                        .and_then(|t| t.as_integer().and_then(|i| i.try_into().ok()))
                        .unwrap_or(60);
                    PrometheusBuilder::new()
                        .with_push_gateway(endpoint, Duration::from_secs(interval))
                        .expect("failed to configure the exporter to run in push gateway mode")
                }
                None => {
                    let host = metrics
                        .get("host")
                        .and_then(|t| t.as_str())
                        .unwrap_or("127.0.0.1");
                    let port = metrics
                        .get("port")
                        .and_then(|t| t.as_integer())
                        .and_then(|t| u16::try_from(t).ok())
                        .unwrap_or(9000);
                    let host_addr = host
                        .parse::<IpAddr>()
                        .unwrap_or_else(|err| panic!("invalid host address `{host}`: {err}"));
                    PrometheusBuilder::new().with_http_listener((host_addr, port))
                }
            };
            if let Some(quantiles) = metrics.get("quantiles").and_then(|t| t.as_array()) {
                let quantiles = quantiles
                    .iter()
                    .filter_map(|q| q.as_float())
                    .collect::<Vec<_>>();
                builder = builder
                    .set_quantiles(&quantiles)
                    .expect("invalid quantiles to render histograms");
            }
            if let Some(buckets) = metrics.get("buckets").and_then(|t| t.as_table()) {
                for (key, value) in buckets {
                    let matcher = if key.starts_with('^') {
                        Matcher::Prefix(key.to_string())
                    } else if key.ends_with('$') {
                        Matcher::Suffix(key.to_string())
                    } else {
                        Matcher::Full(key.to_string())
                    };
                    let values = value
                        .as_array()
                        .expect("buckets should be an array of floats")
                        .iter()
                        .filter_map(|v| v.as_float())
                        .collect::<Vec<_>>();
                    builder = builder
                        .set_buckets_for_metric(matcher, &values)
                        .expect("invalid buckets to render histograms");
                }
            }
            if let Some(labels) = metrics.get("global-labels").and_then(|t| t.as_table()) {
                for (key, value) in labels {
                    builder = builder.add_global_label(key, value.to_string());
                }
            }
            if let Some(addresses) = metrics.get("allowed-addresses").and_then(|t| t.as_array()) {
                for addr in addresses {
                    builder = builder
                        .add_allowed_address(addr.as_str().unwrap_or_default())
                        .unwrap_or_else(|err| panic!("invalid IP address `{addr}`: {err}"));
                }
            }
            builder
                .install()
                .expect("failed to install Prometheus exporter");
        } else if exporter == "tcp" {
            let host = metrics
                .get("host")
                .and_then(|t| t.as_str())
                .unwrap_or("127.0.0.1");
            let port = metrics
                .get("port")
                .and_then(|t| t.as_integer())
                .and_then(|t| u16::try_from(t).ok())
                .unwrap_or(9000);
            let buffer_size = metrics
                .get("buffer_size")
                .and_then(|t| t.as_integer())
                .and_then(|t| usize::try_from(t).ok())
                .unwrap_or(1024);
            let host_addr = host
                .parse::<IpAddr>()
                .unwrap_or_else(|err| panic!("invalid host address `{host}`: {err}"));
            TcpBuilder::new()
                .listen_address((host_addr, port))
                .buffer_size(Some(buffer_size))
                .install()
                .expect("failed to install TCP exporter");
        }
    }
}
