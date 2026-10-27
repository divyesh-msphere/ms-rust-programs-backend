use crate::application::errors::application_error::ApplicationError;
use crate::config::StorageConfig;
use crate::domain::object_storage::{ObjectInfo, ObjectStorage, PresignedRequest};
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use ms_telemetry_rs::tracing::error;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sha2::{Digest, Sha256};
use std::time::Duration as StdDuration;

type HmacSha256 = Hmac<Sha256>;

/// RFC 3986 unreserved characters stay as they are; everything else is %-encoded (AWS Signature V4).
const ENCODE: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'_').remove(b'.').remove(b'~');

fn encode(value: &str) -> String {
    utf8_percent_encode(value, ENCODE).to_string()
}

fn hmac(key: &[u8], data: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// What a presigned URL is built from.
struct Signing<'a> {
    method: &'a str,
    /// `scheme://host[:port]`, no trailing slash.
    base_url: &'a str,
    /// The host header the request will carry (signed).
    host: &'a str,
    /// The URL path, already encoded segment by segment.
    path: &'a str,
    region: &'a str,
    access_key: &'a str,
    secret_key: &'a str,
    expires_seconds: u64,
    /// Headers the caller must send; they are signed in (names in lower case).
    headers: &'a [(String, String)],
    /// Extra query parameters (for example `response-content-disposition`).
    query: &'a [(String, String)],
    now: DateTime<Utc>,
}

/// AWS Signature Version 4, query-string form ("presigned URL"), payload left unsigned.
fn presign_url(s: &Signing) -> String {
    let date = s.now.format("%Y%m%d").to_string();
    let stamp = s.now.format("%Y%m%dT%H%M%SZ").to_string();
    let scope = format!("{date}/{}/s3/aws4_request", s.region);

    let mut headers: Vec<(String, String)> = vec![("host".to_string(), s.host.to_string())];
    headers.extend(s.headers.iter().map(|(k, v)| (k.to_lowercase(), v.trim().to_string())));
    headers.sort();
    let signed_headers = headers.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(";");
    let canonical_headers: String = headers.iter().map(|(k, v)| format!("{k}:{v}\n")).collect();

    let mut query: Vec<(String, String)> = vec![
        ("X-Amz-Algorithm".into(), "AWS4-HMAC-SHA256".into()),
        ("X-Amz-Credential".into(), format!("{}/{scope}", s.access_key)),
        ("X-Amz-Date".into(), stamp.clone()),
        ("X-Amz-Expires".into(), s.expires_seconds.to_string()),
        ("X-Amz-SignedHeaders".into(), signed_headers.clone()),
    ];
    query.extend(s.query.iter().cloned());
    let mut query: Vec<(String, String)> = query.into_iter().map(|(k, v)| (encode(&k), encode(&v))).collect();
    query.sort();
    let canonical_query = query.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join("&");

    let canonical_request =
        format!("{}\n{}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\nUNSIGNED-PAYLOAD", s.method, s.path);
    let string_to_sign = format!("AWS4-HMAC-SHA256\n{stamp}\n{scope}\n{}", hex::encode(Sha256::digest(canonical_request.as_bytes())));

    let k_date = hmac(format!("AWS4{}", s.secret_key).as_bytes(), &date);
    let k_region = hmac(&k_date, s.region);
    let k_service = hmac(&k_region, "s3");
    let k_signing = hmac(&k_service, "aws4_request");
    let signature = hex::encode(hmac(&k_signing, &string_to_sign));

    format!("{}{}?{canonical_query}&X-Amz-Signature={signature}", s.base_url, s.path)
}

/// S3 (or MinIO) over HTTP with presigned URLs: no SDK, no credentials in the browser.
pub struct S3Client {
    config: StorageConfig,
    http: reqwest::Client,
}

impl S3Client {
    pub fn new(config: &StorageConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(StdDuration::from_millis(config.timeout_ms))
            .build()
            .expect("the HTTP client for object storage");
        Self { config: config.clone(), http }
    }

    fn public_endpoint(&self) -> &str {
        let e = if self.config.public_endpoint.trim().is_empty() { &self.config.endpoint } else { &self.config.public_endpoint };
        e.trim_end_matches('/')
    }

    /// `/bucket/key`, each segment encoded, the slashes kept.
    fn object_path(&self, key: &str) -> String {
        let segments: Vec<String> = key.split('/').map(encode).collect();
        format!("/{}/{}", encode(&self.config.bucket), segments.join("/"))
    }

    fn sign(&self, base_url: &str, method: &str, key: &str, headers: &[(String, String)], query: &[(String, String)], now: DateTime<Utc>) -> Result<String, ApplicationError> {
        let url = reqwest::Url::parse(base_url).map_err(|e| ApplicationError::internal_error_with_message(Some(e), "storage endpoint is not a URL"))?;
        let host = match (url.host_str(), url.port()) {
            (Some(h), Some(p)) => format!("{h}:{p}"),
            (Some(h), None) => h.to_string(),
            _ => return Err(ApplicationError::internal_error_with_message(None::<std::io::Error>, "storage endpoint has no host")),
        };
        let path = self.object_path(key);
        Ok(presign_url(&Signing {
            method,
            base_url,
            host: &host,
            path: &path,
            region: &self.config.region,
            access_key: &self.config.access_key,
            secret_key: &self.config.secret_key,
            expires_seconds: self.config.url_expiry_seconds,
            headers,
            query,
            now,
        }))
    }
}

#[async_trait]
impl ObjectStorage for S3Client {
    fn presign_upload(
        &self,
        key: &str,
        content_type: &str,
        size_bytes: i64,
        checksum_sha256_hex: Option<&str>,
    ) -> Result<PresignedRequest, ApplicationError> {
        let now = Utc::now();
        // Type, size and checksum are signed in: the storage refuses an upload that differs.
        let mut headers = vec![
            ("content-type".to_string(), content_type.to_string()),
            ("content-length".to_string(), size_bytes.to_string()),
        ];
        if let Some(hex_sum) = checksum_sha256_hex {
            let bytes = hex::decode(hex_sum).map_err(|_| ApplicationError::unprocessable_entity("checksum must be a SHA-256 in hex"))?;
            headers.push(("x-amz-checksum-sha256".to_string(), BASE64.encode(bytes)));
        }
        let url = self.sign(self.public_endpoint(), "PUT", key, &headers, &[], now)?;
        Ok(PresignedRequest {
            url,
            method: "PUT".to_string(),
            // The browser sets Content-Length itself from the body, so it is not listed.
            headers: headers.into_iter().filter(|(k, _)| k != "content-length").collect(),
            expires_time: now + Duration::seconds(self.config.url_expiry_seconds as i64),
        })
    }

    fn presign_download(&self, key: &str, file_name: &str, content_type: &str) -> Result<PresignedRequest, ApplicationError> {
        let now = Utc::now();
        let disposition = format!("attachment; filename=\"{}\"; filename*=UTF-8''{}", file_name.replace('"', ""), encode(file_name));
        let query = vec![
            ("response-content-disposition".to_string(), disposition),
            ("response-content-type".to_string(), content_type.to_string()),
        ];
        let url = self.sign(self.public_endpoint(), "GET", key, &[], &query, now)?;
        Ok(PresignedRequest {
            url,
            method: "GET".to_string(),
            headers: Vec::new(),
            expires_time: now + Duration::seconds(self.config.url_expiry_seconds as i64),
        })
    }

    async fn head(&self, key: &str) -> Result<Option<ObjectInfo>, ApplicationError> {
        let headers = vec![("x-amz-checksum-mode".to_string(), "ENABLED".to_string())];
        let url = self.sign(self.config.endpoint.trim_end_matches('/'), "HEAD", key, &headers, &[], Utc::now())?;
        let response = self.http.head(&url).header("x-amz-checksum-mode", "ENABLED").send().await.map_err(|e| {
            error!("object storage unreachable: {e:?}");
            ApplicationError::internal_error_with_message(Some(e), "object storage is unreachable")
        })?;
        match response.status().as_u16() {
            404 => Ok(None),
            200 => {
                let header = |name: &str| response.headers().get(name).and_then(|v| v.to_str().ok()).map(String::from);
                Ok(Some(ObjectInfo {
                    size_bytes: header("content-length").and_then(|v| v.parse().ok()).unwrap_or(0),
                    content_type: header("content-type"),
                    checksum_sha256: header("x-amz-checksum-sha256").and_then(|b64| BASE64.decode(b64).ok()).map(hex::encode),
                }))
            }
            status => {
                error!("object storage answered {status} to a HEAD");
                Err(ApplicationError::internal_error_with_message(None::<std::io::Error>, "object storage refused the request"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// The worked example in the AWS documentation ("Authenticating Requests: Using Query Parameters").
    #[test]
    fn matches_the_aws_documented_presigned_get() {
        let url = presign_url(&Signing {
            method: "GET",
            base_url: "https://examplebucket.s3.amazonaws.com",
            host: "examplebucket.s3.amazonaws.com",
            path: "/test.txt",
            region: "us-east-1",
            access_key: "AKIAIOSFODNN7EXAMPLE",
            secret_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            expires_seconds: 86400,
            headers: &[],
            query: &[],
            now: Utc.with_ymd_and_hms(2013, 5, 24, 0, 0, 0).unwrap(),
        });
        assert!(url.ends_with("X-Amz-Signature=aeeed9bbccd4d02ee5c0109b86d86835f995330da4c265957d157751f604d404"), "{url}");
        assert!(url.contains("X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20130524%2Fus-east-1%2Fs3%2Faws4_request"));
        assert!(url.contains("X-Amz-SignedHeaders=host"));
    }

    fn client() -> S3Client {
        S3Client::new(&StorageConfig { access_key: "ak".into(), secret_key: "sk".into(), ..StorageConfig::default() })
    }

    #[test]
    fn upload_signs_type_size_and_checksum() {
        let sum = "00".repeat(32);
        let req = client().presign_upload("programs/p/documents/d", "application/pdf", 1234, Some(&sum)).unwrap();
        assert_eq!(req.method, "PUT");
        assert!(req.url.starts_with("http://localhost:9000/programs-documents/programs/p/documents/d?"));
        assert!(req.url.contains("X-Amz-SignedHeaders=content-length%3Bcontent-type%3Bhost%3Bx-amz-checksum-sha256"), "{}", req.url);
        assert!(req.headers.iter().any(|(k, v)| k == "content-type" && v == "application/pdf"));
        assert!(req.headers.iter().any(|(k, _)| k == "x-amz-checksum-sha256"));
        assert!(!req.headers.iter().any(|(k, _)| k == "content-length"), "the browser sets it from the body");
        assert!(req.expires_time > Utc::now());
    }

    #[test]
    fn upload_refuses_a_checksum_that_is_not_hex() {
        assert!(client().presign_upload("k", "text/plain", 1, Some("not hex")).is_err());
    }

    #[test]
    fn keys_are_encoded_per_segment_and_names_cannot_break_out() {
        let c = client();
        assert_eq!(c.object_path("programs/a b/documents/c"), "/programs-documents/programs/a%20b/documents/c");
        let req = c.presign_download("programs/p/documents/d", "a \"b\"/../c.pdf", "application/pdf").unwrap();
        assert!(req.url.contains("response-content-disposition="));
        assert!(!req.url.contains("../"), "{}", req.url);
    }

    #[test]
    fn same_inputs_sign_the_same_and_different_inputs_do_not() {
        let now = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        let c = client();
        let a = c.sign("http://localhost:9000", "GET", "k1", &[], &[], now).unwrap();
        assert_eq!(a, c.sign("http://localhost:9000", "GET", "k1", &[], &[], now).unwrap());
        assert_ne!(a, c.sign("http://localhost:9000", "GET", "k2", &[], &[], now).unwrap());
        assert_ne!(a, c.sign("http://localhost:9000", "PUT", "k1", &[], &[], now).unwrap());
    }
}
