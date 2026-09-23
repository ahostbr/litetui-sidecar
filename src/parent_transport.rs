//! Dedicated parent pipe reader. Browser pages cannot access this channel.
use std::{
    io::{self, BufRead, Read, Write},
    thread,
};
use tao::event_loop::EventLoopProxy;

use litetui_sidecar::parent_protocol::{self, ParentFrame};

#[derive(Debug)]
pub enum ParentEvent {
    Frame(ParentFrame),
    Disconnected,
}

/// A line-delimited private pipe, with a hard per-frame cap before JSON parsing.
/// Error or EOF means the child exits rather than remaining orphaned.
pub fn read_frames<R: BufRead>(
    mut input: R,
    capability: &str,
    mut deliver: impl FnMut(ParentEvent),
) {
    loop {
        let mut raw = Vec::new();
        let mut limited = Read::take(&mut input, (parent_protocol::MAX_FRAME_BYTES + 2) as u64);
        match limited.read_until(b'\n', &mut raw) {
            Ok(0) | Err(_) => break,
            Ok(_)
                if raw.len() > parent_protocol::MAX_FRAME_BYTES + 1
                    || raw.last() != Some(&b'\n') =>
            {
                break
            }
            Ok(_) => {
                raw.pop();
                if raw.last() == Some(&b'\r') {
                    raw.pop();
                }
                if let Ok(frame) = parent_protocol::decode(&raw, capability) {
                    deliver(ParentEvent::Frame(frame));
                } else {
                    break;
                }
            }
        }
    }
    deliver(ParentEvent::Disconnected);
}

pub fn start(proxy: EventLoopProxy<crate::UserEvent>, capability: String) {
    thread::spawn(move || {
        read_frames(
            io::BufReader::new(io::stdin().lock()),
            &capability,
            |event| {
                let _ = proxy.send_event(crate::UserEvent::Parent(event));
            },
        );
    });
}

pub fn reply(frame: &ParentFrame, data: serde_json::Value) -> io::Result<()> {
    let raw = serde_json::json!({"version": parent_protocol::VERSION, "id": frame.id, "token": frame.token, "command": "reply", "payload": data});
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &raw)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_valid_frame_and_disconnects_on_eof() {
        let mut seen = Vec::new();
        read_frames(
            &b"{\"version\":1,\"id\":7,\"token\":\"cap\",\"command\":\"hello\"}\n"[..],
            "cap",
            |e| seen.push(e),
        );
        assert!(
            matches!(seen.as_slice(), [ParentEvent::Frame(f), ParentEvent::Disconnected] if f.id == 7)
        );
    }

    #[test]
    fn rejects_unterminated_or_oversized_frame() {
        for raw in [
            b"{}".to_vec(),
            vec![b'x'; parent_protocol::MAX_FRAME_BYTES + 3],
        ] {
            let mut seen = Vec::new();
            read_frames(raw.as_slice(), "cap", |e| seen.push(e));
            assert!(matches!(seen.as_slice(), [ParentEvent::Disconnected]));
        }
    }
}
