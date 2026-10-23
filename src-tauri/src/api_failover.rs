use std::time::Duration;

use eyre::{eyre, OptionExt};
use parking_lot::RwLock;
use reqwest::{StatusCode, Url};
use reqwest_middleware::RequestBuilder;

#[derive(Debug)]
pub struct ApiResponse {
    pub status: StatusCode,
    pub body: String,
}

#[derive(Default)]
struct Selection {
    domains: Vec<String>,
    preferred: Option<String>,
    generation: u64,
}

#[derive(Default)]
pub struct ApiFailover {
    selection: RwLock<Selection>,
}

impl ApiFailover {
    pub fn reset(&self) {
        let mut selection = self.selection.write();
        selection.preferred = None;
        selection.generation = selection.generation.wrapping_add(1);
    }

    pub async fn send(
        &self,
        request: RequestBuilder,
        domains: &[String],
        allow_failover: bool,
    ) -> eyre::Result<ApiResponse> {
        if domains.is_empty() {
            return Err(eyre!("没有可用的API线路"));
        }
        let (client, request) = request.build_split();
        let request = request?;
        // This endpoint returns an HTML/JavaScript template rather than JmResp.
        let expects_json = request.url().path() != "/chapter_view_template";
        let (candidates, generation) = {
            let mut selection = self.selection.write();
            if selection.domains != domains {
                selection.domains = domains.to_vec();
                selection.preferred = None;
                selection.generation = selection.generation.wrapping_add(1);
            }
            let mut candidates = domains.to_vec();
            if let Some(index) = selection
                .preferred
                .as_ref()
                .and_then(|preferred| candidates.iter().position(|domain| domain == preferred))
            {
                let preferred = candidates.remove(index);
                candidates.insert(0, preferred);
            }
            if !allow_failover {
                candidates.truncate(1);
            }
            (candidates, selection.generation)
        };

        let mut failures = Vec::new();
        for domain in candidates {
            let mut next = request.try_clone().ok_or_eyre("无法复制API请求")?;
            let mut url = Url::parse(&domain)?;
            url.set_path(request.url().path());
            url.set_query(request.url().query());
            *next.url_mut() = url;

            // Bound the entire attempt, including middleware retries and reading the body.
            let mut response_status = None;
            let attempt = tokio::time::timeout(Duration::from_secs(15), async {
                let response = client.execute(next).await?;
                let status = response.status();
                response_status = Some(status);
                let body = response.text().await?;
                Ok::<_, eyre::Report>(ApiResponse { status, body })
            })
            .await;

            let reason = match attempt {
                Ok(Ok(response)) => {
                    let code = api_code(&response.body);
                    let unavailable =
                        response.status.is_server_error() || (expects_json && code.is_none());
                    // Authentication and rate limits are not reasons to rotate hosts.
                    let stop =
                        !allow_failover || is_protected_status(response.status) || !unavailable;
                    if stop {
                        if response.status == StatusCode::OK && code == Some(200) {
                            let mut selection = self.selection.write();
                            // A late request must not undo a manual selection/proxy change.
                            if selection.generation == generation {
                                if selection.preferred.as_ref() != Some(&domain)
                                    && domain != domains[0]
                                {
                                    tracing::info!(api_domain = %domain, "已自动切换API线路，登录状态可能需要重新验证");
                                }
                                selection.preferred = Some(domain);
                            }
                        }
                        return Ok(response);
                    }
                    format!("HTTP {}，响应不是可用的API数据", response.status)
                }
                Ok(Err(_)) => "连接或读取响应失败".to_string(),
                Err(_) => "请求超时".to_string(),
            };
            // Headers still forbid switching hosts when reading the body fails or stalls.
            if response_status.is_some_and(is_protected_status) {
                return Err(eyre!(
                    "API线路 {domain} 返回 HTTP {}，{reason}",
                    response_status.unwrap()
                ));
            }
            // Do not put request parameters, credentials or response bodies in logs.
            tracing::warn!(api_domain = %domain, reason = %reason, "API线路不可用");
            failures.push(format!("{domain}: {reason}"));
        }
        Err(eyre!("所有API线路均不可用：{}", failures.join("; ")))
    }
}

fn is_protected_status(status: StatusCode) -> bool {
    status == StatusCode::UNAUTHORIZED
        || status == StatusCode::FORBIDDEN
        || status == StatusCode::TOO_MANY_REQUESTS
}

fn api_code(body: &str) -> Option<i64> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let code = value.get("code")?.as_i64()?;
    if code == 200 {
        value.get("data")?;
    }
    Some(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::task::JoinHandle;

    const OK: &str = r#"{"code":200,"data":"encrypted-data"}"#;
    const UNAUTHORIZED: &str = r#"{"code":401,"data":[],"errorMsg":"Invalid credentials"}"#;
    const HTML: &str = "<!DOCTYPE html><script>location.href='/lander'</script>";

    struct Server {
        url: String,
        requests: Arc<Mutex<Vec<String>>>,
        worker: JoinHandle<()>,
    }

    impl Server {
        fn start(status: u16, body: &'static str) -> Self {
            Self::with_delay(status, body, Duration::ZERO)
        }

        fn with_delay(status: u16, body: &'static str, delay: Duration) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            listener.set_nonblocking(true).unwrap();
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let received = requests.clone();
            let worker = tokio::spawn(async move {
                while let Ok((mut stream, _)) = listener.accept().await {
                    let mut data = Vec::new();
                    let mut chunk = [0; 4096];
                    loop {
                        let n = stream.read(&mut chunk).await.unwrap();
                        if n == 0 {
                            break;
                        }
                        data.extend_from_slice(&chunk[..n]);
                        if let Some(end) = data.windows(4).position(|s| s == b"\r\n\r\n") {
                            let header = String::from_utf8_lossy(&data[..end]);
                            let length = header
                                .lines()
                                .find_map(|line| {
                                    let (name, value) = line.split_once(':')?;
                                    name.eq_ignore_ascii_case("content-length")
                                        .then(|| value.trim().parse::<usize>().ok())
                                        .flatten()
                                })
                                .unwrap_or(0);
                            if data.len() >= end + 4 + length {
                                break;
                            }
                        }
                    }
                    if data.is_empty() {
                        continue;
                    }
                    received
                        .lock()
                        .unwrap()
                        .push(String::from_utf8_lossy(&data).into_owned());
                    let response = format!(
                        "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    tokio::time::sleep(delay).await;
                    let _ = stream.write_all(body.as_bytes()).await;
                    let _ = stream.shutdown().await;
                }
            });
            Self {
                url,
                requests,
                worker,
            }
        }

        fn count(&self) -> usize {
            self.requests.lock().unwrap().len()
        }
    }

    impl Drop for Server {
        fn drop(&mut self) {
            self.worker.abort();
        }
    }

    fn client() -> reqwest_middleware::ClientWithMiddleware {
        reqwest_middleware::ClientBuilder::new(
            reqwest::Client::builder().no_proxy().build().unwrap(),
        )
        .build()
    }

    #[tokio::test]
    async fn response_body_timeout_tries_the_next_domain() {
        let slow = Server::with_delay(200, OK, Duration::from_secs(1));
        let good = Server::start(200, OK);
        let response = ApiFailover::default()
            .send(
                client()
                    .get(format!("{}/search", slow.url))
                    .timeout(Duration::from_millis(100)),
                &[slow.url.clone(), good.url.clone()],
                true,
            )
            .await
            .unwrap();
        assert_eq!(response.body, OK);
        assert_eq!(slow.count(), 1);
        assert_eq!(good.count(), 1);
    }

    #[tokio::test]
    async fn protected_status_does_not_fail_over_when_its_body_times_out() {
        for status in [401, 403, 429] {
            let rejected = Server::with_delay(status, "Rejected", Duration::from_secs(1));
            let other = Server::start(200, OK);
            let response = ApiFailover::default()
                .send(
                    client()
                        .get(format!("{}/search", rejected.url))
                        .timeout(Duration::from_millis(100)),
                    &[rejected.url.clone(), other.url.clone()],
                    true,
                )
                .await;
            assert!(response.is_err());
            assert_eq!(other.count(), 0);
        }
    }

    #[tokio::test]
    async fn late_success_does_not_restore_preference_after_reset() {
        let bad = Server::start(200, HTML);
        let good = Server::with_delay(200, OK, Duration::from_millis(50));
        let domains = vec![bad.url.clone(), good.url.clone()];
        let transport = ApiFailover::default();
        let client = client();
        let request = transport.send(client.get(format!("{}/search", bad.url)), &domains, true);
        let reset = async {
            while good.count() == 0 {
                tokio::task::yield_now().await;
            }
            transport.reset();
        };
        let (response, ()) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(request, reset)
        })
        .await
        .unwrap();
        response.unwrap();
        transport
            .send(client.get(format!("{}/search", bad.url)), &domains, true)
            .await
            .unwrap();
        assert_eq!(bad.count(), 2);
    }

    #[tokio::test]
    async fn business_error_without_data_is_not_retried_on_other_domains() {
        let error = r#"{"code":401,"errorMsg":"Invalid credentials"}"#;
        let rejected = Server::start(200, error);
        let other = Server::start(200, OK);
        let response = ApiFailover::default()
            .send(
                client().post(format!("{}/login", rejected.url)),
                &[rejected.url.clone(), other.url.clone()],
                true,
            )
            .await
            .unwrap();
        assert_eq!(response.body, error);
        assert_eq!(other.count(), 0);
    }

    #[tokio::test]
    async fn forbidden_response_is_returned_without_rotating_hosts() {
        let rejected = Server::start(403, "Forbidden");
        let other = Server::start(200, OK);
        let response = ApiFailover::default()
            .send(
                client().get(format!("{}/search", rejected.url)),
                &[rejected.url.clone(), other.url.clone()],
                true,
            )
            .await
            .unwrap();
        assert_eq!(response.status, StatusCode::FORBIDDEN);
        assert_eq!(other.count(), 0);
    }

    #[tokio::test]
    async fn chapter_template_keeps_its_expected_html_response() {
        let template = "<html><script>var scramble_id = 220980;</script></html>";
        let selected = Server::start(200, template);
        let other = Server::start(200, OK);
        let response = ApiFailover::default()
            .send(
                client().get(format!("{}/chapter_view_template", selected.url)),
                &[selected.url.clone(), other.url.clone()],
                true,
            )
            .await
            .unwrap();
        assert_eq!(response.body, template);
        assert_eq!(other.count(), 0);
    }

    #[tokio::test]
    async fn html_response_fails_over_and_next_request_reuses_the_working_domain() {
        let bad = Server::start(200, HTML);
        let good = Server::start(200, OK);
        let domains = vec![bad.url.clone(), good.url.clone()];
        let transport = ApiFailover::default();
        let client = client();
        for _ in 0..2 {
            let response = transport
                .send(client.get(format!("{}/search", bad.url)), &domains, true)
                .await
                .unwrap();
            assert_eq!(response.body, OK);
        }
        assert_eq!(bad.count(), 1);
        assert_eq!(good.count(), 2);
    }

    #[tokio::test]
    async fn connection_failure_tries_the_next_domain() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let offline = format!("http://{}", listener.local_addr().unwrap());
        let good = Server::start(200, OK);
        drop(listener);
        let response = ApiFailover::default()
            .send(
                client().get(format!("{offline}/search")),
                &[offline, good.url.clone()],
                true,
            )
            .await
            .unwrap();
        assert_eq!(response.body, OK);
        assert_eq!(good.count(), 1);
    }

    #[tokio::test]
    async fn invalid_credentials_are_returned_without_trying_other_domains() {
        let rejected = Server::start(401, UNAUTHORIZED);
        let other = Server::start(200, OK);
        let response = ApiFailover::default()
            .send(
                client().post(format!("{}/login", rejected.url)),
                &[rejected.url.clone(), other.url.clone()],
                true,
            )
            .await
            .unwrap();
        assert_eq!(response.status, reqwest::StatusCode::UNAUTHORIZED);
        assert_eq!(response.body, UNAUTHORIZED);
        assert_eq!(other.count(), 0);
    }

    #[tokio::test]
    async fn business_error_does_not_make_a_domain_preferred() {
        let bad = Server::start(200, HTML);
        let rejected = Server::start(200, UNAUTHORIZED);
        let domains = vec![bad.url.clone(), rejected.url.clone()];
        let transport = ApiFailover::default();
        for _ in 0..2 {
            let response = transport
                .send(client().get(format!("{}/search", bad.url)), &domains, true)
                .await
                .unwrap();
            assert_eq!(response.body, UNAUTHORIZED);
        }
        assert_eq!(bad.count(), 2);
    }

    #[tokio::test]
    async fn state_changing_request_is_not_replayed_after_server_failure() {
        let bad = Server::start(503, HTML);
        let other = Server::start(200, OK);
        let response = ApiFailover::default()
            .send(
                client()
                    .post(format!("{}/favorite", bad.url))
                    .form(&[("aid", "1")]),
                &[bad.url.clone(), other.url.clone()],
                false,
            )
            .await
            .unwrap();
        assert_eq!(response.status, reqwest::StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(bad.count(), 1);
        assert_eq!(other.count(), 0);
    }

    #[tokio::test]
    async fn rate_limit_is_not_bypassed_by_switching_domains() {
        let limited = Server::start(429, "Too many requests");
        let other = Server::start(200, OK);
        let response = ApiFailover::default()
            .send(
                client().get(format!("{}/search", limited.url)),
                &[limited.url.clone(), other.url.clone()],
                true,
            )
            .await
            .unwrap();
        assert_eq!(response.status, reqwest::StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(other.count(), 0);
    }

    #[tokio::test]
    async fn server_error_and_unrelated_json_both_trigger_failover() {
        let unavailable = Server::start(503, OK);
        let unrelated = Server::start(200, r#"{"message":"not an API response"}"#);
        let good = Server::start(200, OK);
        let domains = vec![
            unavailable.url.clone(),
            unrelated.url.clone(),
            good.url.clone(),
        ];
        let response = ApiFailover::default()
            .send(
                client().get(format!("{}/search", unavailable.url)),
                &domains,
                true,
            )
            .await
            .unwrap();
        assert_eq!(response.body, OK);
        assert_eq!(unavailable.count(), 1);
        assert_eq!(unrelated.count(), 1);
        assert_eq!(good.count(), 1);
    }

    #[tokio::test]
    async fn custom_domain_does_not_reuse_a_cached_builtin_fallback() {
        let selected = Server::start(200, HTML);
        let good = Server::start(200, OK);
        let transport = ApiFailover::default();
        let builder = || client().get(format!("{}/search", selected.url));
        transport
            .send(builder(), &[selected.url.clone(), good.url.clone()], true)
            .await
            .unwrap();
        assert!(transport
            .send(builder(), &[selected.url.clone()], true)
            .await
            .is_err());
        assert_eq!(selected.count(), 2);
        assert_eq!(good.count(), 1);
    }

    #[tokio::test]
    async fn resetting_after_proxy_change_retries_the_selected_domain() {
        let bad = Server::start(200, HTML);
        let good = Server::start(200, OK);
        let domains = vec![bad.url.clone(), good.url.clone()];
        let transport = ApiFailover::default();
        transport
            .send(client().get(format!("{}/search", bad.url)), &domains, true)
            .await
            .unwrap();
        transport.reset();
        transport
            .send(client().get(format!("{}/search", bad.url)), &domains, true)
            .await
            .unwrap();
        assert_eq!(bad.count(), 2);
    }

    #[tokio::test]
    async fn exhausted_domains_are_each_attempted_once_and_reported() {
        let first = Server::start(200, HTML);
        let second = Server::start(503, "Unavailable");
        let error = ApiFailover::default()
            .send(
                client().get(format!("{}/search", first.url)),
                &[first.url.clone(), second.url.clone()],
                true,
            )
            .await
            .unwrap_err();
        let message = error.to_string();
        assert!(message.contains(&first.url));
        assert!(message.contains(&second.url));
        assert!(!message.contains(HTML));
        assert_eq!(first.count(), 1);
        assert_eq!(second.count(), 1);
    }

    #[tokio::test]
    async fn failover_preserves_path_query_headers_and_login_form() {
        let bad = Server::start(200, HTML);
        let good = Server::start(200, OK);
        let request = client()
            .post(format!("{}/login?page=1", bad.url))
            .header("token", "test-token")
            .form(&[("username", "test"), ("password", "dummy-value")]);
        ApiFailover::default()
            .send(request, &[bad.url.clone(), good.url.clone()], true)
            .await
            .unwrap();
        let requests = good.requests.lock().unwrap();
        assert!(requests[0].starts_with("POST /login?page=1 HTTP/1.1\r\n"));
        assert!(requests[0].contains("token: test-token\r\n"));
        assert!(requests[0].ends_with("username=test&password=dummy-value"));
    }
}
