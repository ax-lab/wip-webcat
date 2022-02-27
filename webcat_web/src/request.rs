use std::time::Duration;

use super::Response;
use reqwest::Url;

/// Provides support for setting up and executing HTTP requests.
pub struct Request {
	timeout: Option<Duration>,
}

impl Request {
	pub fn new() -> Self {
		Request { timeout: None }
	}

	pub fn with_timeout(mut self, duration: Duration) -> Self {
		self.timeout = Some(duration);
		self
	}

	pub fn send<T: AsRef<str>>(&self, method: RequestMethod, url: T) -> RequestResult {
		let url = Self::parse_url(url)?;
		let client = reqwest::blocking::Client::new();

		let method = match method {
			RequestMethod::GET => reqwest::Method::GET,
			RequestMethod::POST => reqwest::Method::POST,
		};

		let request = client.request(method, url);
		let request = if let Some(duration) = self.timeout {
			request.timeout(duration)
		} else {
			request
		};
		let response = request
			.send()
			.map_err(|err| RequestError::ConnectionFailed(err.to_string()))?;
		let response = Response::from_reqwest(response);
		Ok(response)
	}

	fn parse_url<S: AsRef<str>>(url: S) -> Result<Url, RequestError> {
		let url = url.as_ref();
		let url = if !(url.starts_with("http://") || url.starts_with("https://")) {
			let url = format!("http://{}", url);
			Url::parse(&url)
		} else {
			Url::parse(url)
		};

		url.map_err(|err| RequestError::InvalidConfiguration(err.to_string()))
	}
}

pub enum RequestMethod {
	GET,
	POST,
}

#[derive(Debug)]
pub enum RequestError {
	ConnectionFailed(String),
	InvalidConfiguration(String),
}

impl std::fmt::Display for RequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			RequestError::ConnectionFailed(msg) => write!(f, "connection error: {}", msg),
			RequestError::InvalidConfiguration(msg) => write!(f, "invalid request: {}", msg),
		}
	}
}

pub type RequestResult = std::result::Result<Response, RequestError>;

#[cfg(test)]
mod tests {
	use super::*;
	use tux::*;

	fn make_get_request_with_path(server: TestServer, path: &str) -> Response {
		let url = format!("http://127.0.0.1:{}/{}", server.port(), path);
		let result = Request::new()
			.send(RequestMethod::GET, url)
			.expect("request failed");
		result
	}

	#[test]
	fn request_returns_response_body() {
		let server = TestServer::new_with_root_response("test server");
		let result = make_get_request_with_path(server, "");
		assert_eq!(result.status_code(), 200);
		assert_eq!(result.text(), "test server");
	}

	#[test]
	fn request_returns_404_for_inexistent_path() {
		let server = TestServer::new_with_root_response("test server");
		let result = make_get_request_with_path(server, "this_path_does_not_exist");
		assert_eq!(result.status_code(), 404);
	}

	#[test]
	fn request_returns_connection_error_for_inexistent_server() {
		let result = Request::new()
			.with_timeout(Duration::from_millis(50))
			.send(RequestMethod::GET, "http://127.0.0.1:753");
		match result {
			Err(RequestError::ConnectionFailed(_)) => {}
			Err(err) => {
				panic!("wrong error: {}", err)
			}
			Ok(_) => {
				panic!("did not fail")
			}
		}
	}

	#[test]
	fn request_returns_configuration_error_for_invalid_url() {
		// Try to setup a request with an invalid URL port
		let result = Request::new().send(RequestMethod::GET, "http://127.0.0.1:99999");
		match result {
			Err(RequestError::InvalidConfiguration(_)) => {}
			Err(err) => {
				panic!("wrong error: {}", err)
			}
			Ok(_) => {
				panic!("did not fail")
			}
		}
	}

	#[test]
	fn request_should_default_to_http() {
		let server = TestServer::new_with_root_response("ok");
		let url = format!("localhost:{}", server.port());
		let result = Request::new()
			.send(RequestMethod::GET, url)
			.expect("request failed");
		assert_eq!(result.text(), "ok");
	}
}
