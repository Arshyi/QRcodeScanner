use crate::{BrowserPort, ClipboardPort, PortError};
use qrforge_domain::{Detection, PayloadClass, classify_payload};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc, Mutex, PoisonError,
    atomic::{AtomicU64, Ordering},
};
use thiserror::Error;

const PREVIEW_CHARACTER_LIMIT: usize = 240;

/// Rust-owned display category for one decoded result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    /// Approved plain HTTP URL.
    HttpUrl,
    /// Approved encrypted HTTPS URL.
    HttpsUrl,
    /// Ordinary UTF-8 text.
    PlainText,
    /// HTTP-like text that did not parse as a URL.
    MalformedUrl,
    /// A syntactically valid URL using a prohibited scheme.
    BlockedScheme,
    /// An HTTP(S) URL blocked because of credentials or a spoofable host.
    BlockedUrl,
    /// Empty or control-character text that cannot be copied.
    UnsafeText,
    /// Non-UTF-8 payload bytes.
    Binary,
}

/// Inert, bounded result metadata sent to the chooser webview.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultItemView {
    /// Original decoder ordering, starting at zero.
    pub index: usize,
    /// Rust-owned security classification.
    pub kind: ResultKind,
    /// Bounded human-readable preview rendered by the webview as text.
    pub preview: String,
    /// Optional non-sensitive category detail such as a blocked scheme.
    pub detail: Option<String>,
    /// Whether Rust permits this item to be opened.
    pub can_open: bool,
    /// Whether Rust permits this item to be copied.
    pub can_copy: bool,
}

/// Current chooser session returned through typed IPC.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingResultsView {
    /// Opaque generation used to reject stale chooser commands.
    pub session_id: u64,
    /// Results in original decoder order.
    pub items: Vec<ResultItemView>,
}

/// Typed chooser action accepted at the IPC boundary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResultActionKind {
    /// Open one Rust-approved HTTP(S) item.
    Open,
    /// Copy one Rust-approved textual item.
    Copy,
    /// Copy every Rust-approved textual item in original order.
    CopyAll,
    /// Discard the pending result set.
    Dismiss,
}

/// Complete result action request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResultActionRequest {
    /// Session returned by [`PendingResultsView`].
    pub session_id: u64,
    /// Requested Rust-side action.
    pub action: ResultActionKind,
    /// Required for single-item actions and forbidden otherwise.
    pub index: Option<usize>,
}

/// User-safe action result returned to the chooser.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultActionOutcome {
    /// Short inert status message.
    pub message: &'static str,
    /// Whether the chooser should close after the action.
    pub close: bool,
}

/// Native-only classified payload retained for explicit chooser actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassifiedResult {
    view: ResultItemView,
    class: PayloadClass,
    copy_text: Option<String>,
}

/// Classifies detections without reordering them or executing any action.
#[must_use]
pub(crate) fn classify_results(detections: &[Detection]) -> Vec<ClassifiedResult> {
    detections
        .iter()
        .enumerate()
        .map(|(index, detection)| ClassifiedResult::new(index, detection))
        .collect()
}

impl ClassifiedResult {
    fn new(index: usize, detection: &Detection) -> Self {
        let class = classify_payload(&detection.raw_bytes);
        let original_text = std::str::from_utf8(&detection.raw_bytes)
            .ok()
            .map(ToOwned::to_owned);
        let (kind, preview, detail, can_open, can_copy) = match &class {
            PayloadClass::SafeUrl(url) => (
                if url.scheme() == "https" {
                    ResultKind::HttpsUrl
                } else {
                    ResultKind::HttpUrl
                },
                preview_text(original_text.as_deref().unwrap_or(url.as_str())),
                None,
                true,
                true,
            ),
            PayloadClass::PlainText(text) => {
                (ResultKind::PlainText, preview_text(text), None, false, true)
            }
            PayloadClass::MalformedUrl { text } => (
                ResultKind::MalformedUrl,
                preview_text(text),
                Some("Malformed HTTP-like text".to_owned()),
                false,
                true,
            ),
            PayloadClass::BlockedScheme { scheme, text } => (
                ResultKind::BlockedScheme,
                preview_text(text),
                Some(format!("Blocked {scheme} scheme")),
                false,
                true,
            ),
            PayloadClass::BlockedUrl { text } => (
                ResultKind::BlockedUrl,
                preview_text(text),
                Some("Blocked URL authority".to_owned()),
                false,
                true,
            ),
            PayloadClass::UnsafeText => (
                ResultKind::UnsafeText,
                "Unsafe text content is not shown or copyable".to_owned(),
                None,
                false,
                false,
            ),
            PayloadClass::Binary => (
                ResultKind::Binary,
                format!("Binary QR payload ({} bytes)", detection.raw_bytes.len()),
                None,
                false,
                false,
            ),
        };
        Self {
            view: ResultItemView {
                index,
                kind,
                preview,
                detail,
                can_open,
                can_copy,
            },
            class,
            copy_text: if can_copy { original_text } else { None },
        }
    }
}

struct PendingSession {
    id: u64,
    items: Vec<ClassifiedResult>,
}

/// Native pending-result store and action policy.
pub struct ResultService {
    browser: Arc<dyn BrowserPort>,
    clipboard: Arc<dyn ClipboardPort>,
    next_session: AtomicU64,
    pending: Mutex<Option<PendingSession>>,
}

impl ResultService {
    /// Creates an empty result service over explicit operating-system ports.
    #[must_use]
    pub fn new(browser: Arc<dyn BrowserPort>, clipboard: Arc<dyn ClipboardPort>) -> Self {
        Self {
            browser,
            clipboard,
            next_session: AtomicU64::new(1),
            pending: Mutex::new(None),
        }
    }

    /// Replaces any prior chooser content and returns the new inert view.
    pub fn publish(&self, items: Vec<ClassifiedResult>) -> PendingResultsView {
        let id = self.next_session.fetch_add(1, Ordering::AcqRel);
        let view = PendingResultsView {
            session_id: id,
            items: items.iter().map(|item| item.view.clone()).collect(),
        };
        *self.pending.lock().unwrap_or_else(PoisonError::into_inner) =
            Some(PendingSession { id, items });
        view
    }

    /// Returns the current chooser view without exposing action-capable values.
    #[must_use]
    pub fn snapshot(&self) -> Option<PendingResultsView> {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .map(|pending| PendingResultsView {
                session_id: pending.id,
                items: pending.items.iter().map(|item| item.view.clone()).collect(),
            })
    }

    /// Applies one validated chooser action in Rust.
    pub fn perform(
        &self,
        request: &ResultActionRequest,
    ) -> Result<ResultActionOutcome, ResultActionError> {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        let session = pending
            .as_ref()
            .filter(|session| session.id == request.session_id)
            .ok_or(ResultActionError::StaleSession)?;
        match request.action {
            ResultActionKind::Open => {
                let item = indexed_item(session, request.index)?;
                let PayloadClass::SafeUrl(url) = &item.class else {
                    return Err(ResultActionError::NotOpenable);
                };
                self.browser.open(url).map_err(ResultActionError::Browser)?;
                *pending = None;
                Ok(ResultActionOutcome {
                    message: "Link opened",
                    close: true,
                })
            }
            ResultActionKind::Copy => {
                let item = indexed_item(session, request.index)?;
                let text = item
                    .copy_text
                    .as_deref()
                    .ok_or(ResultActionError::NotCopyable)?;
                self.clipboard
                    .set_text(text)
                    .map_err(ResultActionError::Clipboard)?;
                Ok(ResultActionOutcome {
                    message: "Result copied",
                    close: false,
                })
            }
            ResultActionKind::CopyAll => {
                if request.index.is_some() {
                    return Err(ResultActionError::InvalidRequest);
                }
                let joined = session
                    .items
                    .iter()
                    .filter_map(|item| item.copy_text.as_deref())
                    .collect::<Vec<_>>()
                    .join("\n");
                if joined.is_empty() {
                    return Err(ResultActionError::NotCopyable);
                }
                self.clipboard
                    .set_text(&joined)
                    .map_err(ResultActionError::Clipboard)?;
                Ok(ResultActionOutcome {
                    message: "Copyable results copied in scan order",
                    close: false,
                })
            }
            ResultActionKind::Dismiss => {
                if request.index.is_some() {
                    return Err(ResultActionError::InvalidRequest);
                }
                *pending = None;
                Ok(ResultActionOutcome {
                    message: "Results dismissed",
                    close: true,
                })
            }
        }
    }

    /// Clears a session only if it is still current.
    pub fn clear(&self, session_id: Option<u64>) {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        if session_id.is_none()
            || pending
                .as_ref()
                .is_some_and(|session| Some(session.id) == session_id)
        {
            *pending = None;
        }
    }
}

fn indexed_item(
    session: &PendingSession,
    index: Option<usize>,
) -> Result<&ClassifiedResult, ResultActionError> {
    let index = index.ok_or(ResultActionError::InvalidRequest)?;
    session
        .items
        .get(index)
        .filter(|item| item.view.index == index)
        .ok_or(ResultActionError::InvalidIndex)
}

fn preview_text(text: &str) -> String {
    let mut characters = text.chars();
    let preview = characters
        .by_ref()
        .take(PREVIEW_CHARACTER_LIMIT)
        .collect::<String>();
    if characters.next().is_some() {
        format!("{preview}…")
    } else {
        preview
    }
}

/// Result action rejection with no payload-bearing display text.
#[derive(Debug, Error)]
pub enum ResultActionError {
    /// Chooser data has been replaced or dismissed.
    #[error("result session is no longer available")]
    StaleSession,
    /// Request shape did not match its selected action.
    #[error("result action request is invalid")]
    InvalidRequest,
    /// Requested result index was outside the current ordered set.
    #[error("result index is invalid")]
    InvalidIndex,
    /// Rust policy does not permit browser opening for this classification.
    #[error("result is not approved for opening")]
    NotOpenable,
    /// Rust policy does not permit clipboard copying for this classification.
    #[error("result is not copyable")]
    NotCopyable,
    /// System browser adapter failed.
    #[error("browser action failed")]
    Browser(PortError),
    /// System clipboard adapter failed.
    #[error("clipboard action failed")]
    Clipboard(PortError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrforge_domain::{Point, SafeHttpUrl};

    #[derive(Default)]
    struct Calls {
        opened: Mutex<Vec<String>>,
        copied: Mutex<Vec<String>>,
    }

    impl BrowserPort for Calls {
        fn open(&self, url: &SafeHttpUrl) -> Result<(), PortError> {
            self.opened
                .lock()
                .expect("browser calls")
                .push(url.as_str().to_owned());
            Ok(())
        }
    }

    impl ClipboardPort for Calls {
        fn set_text(&self, text: &str) -> Result<(), PortError> {
            self.copied
                .lock()
                .expect("clipboard calls")
                .push(text.to_owned());
            Ok(())
        }
    }

    fn detection(value: &[u8]) -> Detection {
        Detection::new(
            value.to_vec(),
            qrforge_domain::QrFormat::QrCode,
            [Point { x: 0, y: 0 }; 4],
        )
    }

    #[test]
    fn preserves_order_and_classifies_each_result() {
        let results = classify_results(&[
            detection(b"http://example.com"),
            detection(b"plain text"),
            detection(b"javascript:alert(1)"),
            detection(b"https://[invalid"),
        ]);
        assert_eq!(
            results
                .iter()
                .map(|item| item.view.kind)
                .collect::<Vec<_>>(),
            [
                ResultKind::HttpUrl,
                ResultKind::PlainText,
                ResultKind::BlockedScheme,
                ResultKind::MalformedUrl,
            ]
        );
        assert_eq!(
            results
                .iter()
                .map(|item| item.view.index)
                .collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
    }

    #[test]
    fn opens_only_safe_urls_and_copies_in_original_order() {
        let calls = Arc::new(Calls::default());
        let service = ResultService::new(calls.clone(), calls.clone());
        let session = service.publish(classify_results(&[
            detection(b"https://example.com/one"),
            detection(b"plain text"),
            detection(b"file:///C:/blocked"),
        ]));

        assert!(matches!(
            service.perform(&ResultActionRequest {
                session_id: session.session_id,
                action: ResultActionKind::Open,
                index: Some(2),
            }),
            Err(ResultActionError::NotOpenable)
        ));
        service
            .perform(&ResultActionRequest {
                session_id: session.session_id,
                action: ResultActionKind::CopyAll,
                index: None,
            })
            .expect("copy all");
        assert_eq!(
            calls.copied.lock().expect("clipboard calls").last(),
            Some(&"https://example.com/one\nplain text\nfile:///C:/blocked".to_owned())
        );
        service
            .perform(&ResultActionRequest {
                session_id: session.session_id,
                action: ResultActionKind::Open,
                index: Some(0),
            })
            .expect("safe open");
    }

    #[test]
    fn long_unicode_preview_is_bounded_by_characters() {
        let text = "界".repeat(PREVIEW_CHARACTER_LIMIT + 20);
        let results = classify_results(&[detection(text.as_bytes())]);
        assert_eq!(
            results[0].view.preview.chars().count(),
            PREVIEW_CHARACTER_LIMIT + 1
        );
        assert!(results[0].view.preview.ends_with('…'));
    }

    #[test]
    fn closing_actions_clear_sessions_and_stale_commands_are_rejected() {
        let calls = Arc::new(Calls::default());
        let service = ResultService::new(calls.clone(), calls);
        let first = service.publish(classify_results(&[detection(b"https://example.com")]));
        service
            .perform(&ResultActionRequest {
                session_id: first.session_id,
                action: ResultActionKind::Open,
                index: Some(0),
            })
            .expect("approved link opens");
        assert!(service.snapshot().is_none());
        assert!(matches!(
            service.perform(&ResultActionRequest {
                session_id: first.session_id,
                action: ResultActionKind::Dismiss,
                index: None,
            }),
            Err(ResultActionError::StaleSession)
        ));
    }

    #[test]
    fn binary_and_unsafe_items_never_expose_or_copy_raw_content() {
        let results = classify_results(&[
            detection(&[0xff, 0xfe, 0xfd]),
            detection(b"line one\nline two"),
        ]);
        assert_eq!(results[0].view.kind, ResultKind::Binary);
        assert!(!results[0].view.can_copy);
        assert!(!results[0].view.preview.contains('\u{fffd}'));
        assert_eq!(results[1].view.kind, ResultKind::UnsafeText);
        assert!(!results[1].view.can_copy);
        assert!(!results[1].view.preview.contains("line one"));
    }

    #[test]
    fn action_shape_validation_rejects_extra_or_missing_indexes() {
        let calls = Arc::new(Calls::default());
        let service = ResultService::new(calls.clone(), calls);
        let session = service.publish(classify_results(&[detection(b"plain text")]));
        for request in [
            ResultActionRequest {
                session_id: session.session_id,
                action: ResultActionKind::Copy,
                index: None,
            },
            ResultActionRequest {
                session_id: session.session_id,
                action: ResultActionKind::CopyAll,
                index: Some(0),
            },
        ] {
            assert!(matches!(
                service.perform(&request),
                Err(ResultActionError::InvalidRequest)
            ));
        }
    }

    #[test]
    fn ipc_request_rejects_unknown_fields_and_unknown_actions() {
        assert!(
            serde_json::from_str::<ResultActionRequest>(
                r#"{"sessionId":1,"action":"copy_all","index":null,"extra":true}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<ResultActionRequest>(
                r#"{"sessionId":1,"action":"execute","index":0}"#
            )
            .is_err()
        );
    }
}
