/*
 *  Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *  SPDX-License-Identifier: Apache-2.0
 */

use crate::endpoint_lib::diagnostic::DiagnosticCollector;
use http::Uri;
use url::{Host, Url as ParsedUrl};

#[derive(PartialEq, Debug)]
pub(crate) struct Url<'a> {
    uri: Uri,
    url: ParsedUrl,
    raw: &'a str,
}

// individual methods on parse_url might not be used (although the [`parse_url`] itself _MUST_ be used
// since stdlib functions are pulled into crate lazily)
#[allow(unused)]
impl Url<'_> {
    pub(crate) fn is_ip(&self) -> bool {
        matches!(self.url.host(), Some(Host::Ipv4(_) | Host::Ipv6(_)))
    }
    pub(crate) fn scheme(&self) -> &str {
        self.url.scheme()
    }

    pub(crate) fn authority(&self) -> &str {
        self.uri.authority().unwrap().as_str()
    }

    pub(crate) fn normalized_path(&self) -> &str {
        match self.uri.path() {
            path if !path.is_empty() => path,
            _ => "/",
        }
    }

    pub(crate) fn path(&self) -> &str {
        if self.uri.path() == "/" && !self.raw.ends_with('/') {
            ""
        } else {
            self.uri.path()
        }
    }
}

pub(crate) fn parse_url<'a>(url: &'a str, e: &mut DiagnosticCollector) -> Option<Url<'a>> {
    let raw = url;
    let uri: Uri = e.capture(url.parse())?;
    let url: ParsedUrl = e.capture(url.parse())?;
    if let Some(query) = uri.query() {
        e.report_error(format!(
            "URL cannot have a query component (found {})",
            query
        ));
        return None;
    }
    if !["http", "https"].contains(&url.scheme()) {
        e.report_error(format!(
            "URL scheme must be HTTP or HTTPS (found {})",
            url.scheme()
        ));
        return None;
    }
    Some(Url { url, uri, raw })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::endpoint_lib::diagnostic::DiagnosticCollector;

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn parse_simple_url() {
        let url = "https://control.vpce-1a2b3c4d-5e6f.s3.us-west-2.vpce.amazonaws.com";
        let url = parse_url(url, &mut DiagnosticCollector::new()).expect("valid url");
        assert_eq!(url.path(), "");
        assert_eq!(url.normalized_path(), "/");
        assert_eq!(url.is_ip(), false);
        assert_eq!(url.scheme(), "https");
        assert_eq!(
            url.authority(),
            "control.vpce-1a2b3c4d-5e6f.s3.us-west-2.vpce.amazonaws.com"
        );
    }

    #[test]
    fn schemes_are_normalized() {
        let url = "HTTPS://control.vpce-1a2b3c4d-5e6f.s3.us-west-2.vpce.amazonaws.com";
        let url = parse_url(url, &mut DiagnosticCollector::new()).expect("valid url");
        assert_eq!(url.scheme(), "https");
    }

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn parse_url_with_port() {
        let url = "http://localhost:8000/path";
        let url = parse_url(url, &mut DiagnosticCollector::new()).expect("valid url");
        assert_eq!(url.path(), "/path");
        assert_eq!(url.normalized_path(), "/path");
        assert_eq!(url.is_ip(), false);
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.authority(), "localhost:8000");
    }

    #[test]
    fn only_http_https_supported() {
        let url = "wss://localhost:8443/path";
        assert_eq!(parse_url(url, &mut DiagnosticCollector::new()), None);
    }
}
