//! The parent/child wire envelope. LiteTUI owns data; the WebView never reads files to
//! answer settings or calendar requests. This module validates untrusted line frames.
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const VERSION: u32 = 1;
pub const MAX_FRAME_BYTES: usize = 1_048_576;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ParentFrame {
    pub version: u32,
    pub id: u64,
    pub token: String,
    pub command: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, PartialEq)]
pub enum FrameError {
    TooLarge,
    Malformed,
    WrongVersion,
    WrongToken,
}

pub fn decode(raw: &[u8], expected_token: &str) -> Result<ParentFrame, FrameError> {
    if raw.len() > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge);
    }
    let frame: ParentFrame = serde_json::from_slice(raw).map_err(|_| FrameError::Malformed)?;
    if frame.version != VERSION {
        return Err(FrameError::WrongVersion);
    }
    if expected_token.is_empty() || frame.token != expected_token {
        return Err(FrameError::WrongToken);
    }
    if frame.command.is_empty() {
        return Err(FrameError::Malformed);
    }
    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_wrong_version_token_and_malformed_frames() {
        let valid = br#"{"version":1,"id":7,"token":"secret","command":"open","payload":{"view":"settings"}}"#;
        assert_eq!(decode(valid, "secret").unwrap().id, 7);
        assert_eq!(decode(valid, "other"), Err(FrameError::WrongToken));
        assert_eq!(decode(valid, ""), Err(FrameError::WrongToken));
        assert_eq!(
            decode(
                br#"{"version":2,"id":7,"token":"secret","command":"open"}"#,
                "secret"
            ),
            Err(FrameError::WrongVersion)
        );
        assert_eq!(
            decode(
                br#"{"version":1,"id":7,"token":"secret","command":"open","oops":1}"#,
                "secret"
            ),
            Err(FrameError::Malformed)
        );
        assert_eq!(
            decode(&vec![b'x'; MAX_FRAME_BYTES + 1], "secret"),
            Err(FrameError::TooLarge)
        );
    }
}
