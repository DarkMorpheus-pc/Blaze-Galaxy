use std::io;
use tokio::io::{AsyncBufRead, AsyncBufReadExt};

/// Newline-delimited IPC with bounded memory and cancellation-safe partial reads.
pub struct JsonLines<R> {
    reader: R,
    pending: Vec<u8>,
    max_length: usize,
}

impl<R: AsyncBufRead + Unpin> JsonLines<R> {
    pub fn new(reader: R, max_length: usize) -> Self {
        Self {
            reader,
            pending: Vec::new(),
            max_length,
        }
    }

    pub async fn next_frame(&mut self) -> io::Result<Option<Vec<u8>>> {
        loop {
            let available = self.reader.fill_buf().await?;
            if available.is_empty() {
                return if self.pending.is_empty() {
                    Ok(None)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "incomplete IPC frame",
                    ))
                };
            }
            let newline = available.iter().position(|b| *b == b'\n');
            let count = newline.map_or(available.len(), |i| i + 1);
            if self.pending.len().saturating_add(count) > self.max_length {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "IPC frame too large",
                ));
            }
            self.pending.extend_from_slice(&available[..count]);
            self.reader.consume(count);
            if newline.is_some() {
                return Ok(Some(std::mem::take(&mut self.pending)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, BufReader};

    #[tokio::test]
    async fn cancellation_preserves_fragmented_utf8_and_following_frame() {
        let (mut writer, reader) = tokio::io::duplex(64);
        let mut frames = JsonLines::new(BufReader::new(reader), 64);
        writer.write_all(b"{\"x\":\"\xc3").await.unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(10), frames.next_frame())
                .await
                .is_err()
        );
        writer.write_all(b"\xa7\"}\n{}\n").await.unwrap();
        assert_eq!(
            frames.next_frame().await.unwrap().unwrap(),
            "{\"x\":\"ç\"}\n".as_bytes()
        );
        assert_eq!(frames.next_frame().await.unwrap().unwrap(), b"{}\n");
    }

    #[tokio::test]
    async fn rejects_oversized_and_truncated_frames() {
        let mut frames = JsonLines::new(&b"123456"[..], 4);
        assert_eq!(
            frames.next_frame().await.unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let mut frames = JsonLines::new(&b"{}"[..], 64);
        assert_eq!(
            frames.next_frame().await.unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
}
