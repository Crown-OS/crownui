use std::{io, path::Path};

use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

const TOKEN_PREFIX: &str = "TOKEN ";
const MAX_HANDSHAKE_LINE: u64 = 4096;

/// Connects to an app socket, presenting `token` in the `TOKEN <token>\n` handshake
/// line; the stream is then ready for an MCP client.
pub async fn connect(socket: &Path, token: Option<&str>) -> io::Result<UnixStream> {
    let mut stream = UnixStream::connect(socket).await?;
    if let Some(token) = token {
        if token.contains('\n') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "token contains a newline",
            ));
        }
        stream
            .write_all(format!("{TOKEN_PREFIX}{token}\n").as_bytes())
            .await?;
    }
    Ok(stream)
}

pub(crate) async fn read_token(
    reader: &mut (impl AsyncBufRead + Unpin),
) -> io::Result<Option<String>> {
    if reader.fill_buf().await?.first() != Some(&TOKEN_PREFIX.as_bytes()[0]) {
        return Ok(None);
    }
    let mut line = String::new();
    reader.take(MAX_HANDSHAKE_LINE).read_line(&mut line).await?;
    line.strip_prefix(TOKEN_PREFIX)
        .and_then(|token| token.strip_suffix('\n'))
        .map(|token| Some(token.to_owned()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "malformed TOKEN handshake line"))
}

#[cfg(test)]
mod tests {
    use super::read_token;

    #[tokio::test]
    async fn token_lines_are_consumed_and_json_is_left_alone() {
        let mut with_token: &[u8] = b"TOKEN abc\n{\"jsonrpc\"";
        assert_eq!(
            read_token(&mut with_token).await.ok(),
            Some(Some("abc".into()))
        );
        assert_eq!(with_token, b"{\"jsonrpc\"");
        let mut without_token: &[u8] = b"{\"jsonrpc\"";
        assert_eq!(read_token(&mut without_token).await.ok(), Some(None));
        assert_eq!(without_token, b"{\"jsonrpc\"");
        let mut malformed: &[u8] = b"TOKENabc\n";
        assert!(read_token(&mut malformed).await.is_err());
    }
}
