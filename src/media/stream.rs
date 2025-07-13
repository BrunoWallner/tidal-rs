use std::io;

use reqwest::{Client, Response};

use crate::media::TrackStream;

pub trait Streamable {
    fn get_stream_urls(&self) -> &Vec<String>;
}

impl Streamable for TrackStream {
    fn get_stream_urls(&self) -> &Vec<String> {
        &self.manifest.urls
    }
}

pub struct Stream {
    data: Vec<u8>,
    pos: usize,
    response: Response,
    buffer_size: usize,
}

impl Stream {
    pub async fn init<T: Streamable>(t: T, buffer_size: usize) -> io::Result<Self> {
        let client = Client::new();
        let buffer = Vec::new();
        let pos = 0;
        let Some(url) = t.get_stream_urls().get(0) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "item has no urls available",
            ));
        };
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::ConnectionRefused, e))?;

        Ok(Self {
            data: buffer,
            pos,
            response,
            buffer_size,
        })
    }
}

impl io::Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read = 0;
        let output_size = buf.len();

        let to_read = (self.pos + output_size + self.buffer_size).saturating_sub(self.data.len());
        while read < to_read {
            let streamed_chunk = smol::block_on(self.response.chunk());
            let streamed_chunk =
                streamed_chunk.map_err(|e| io::Error::new(io::ErrorKind::ConnectionReset, e))?;
            let Some(streamed_chunk) = streamed_chunk else {
                // EOF
                break;
            };
            read += streamed_chunk.len();
            self.data.extend_from_slice(&streamed_chunk);
        }
        let available = self.data.len().saturating_sub(self.pos);
        let to_copy = output_size.min(available);
        let copy_start = self.pos;
        let copy_end = copy_start + to_copy;
        buf[..to_copy].copy_from_slice(&self.data[copy_start..copy_end]);
        self.pos += to_copy;

        Ok(to_copy)
    }
}
impl io::Seek for Stream {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let buf_len = self.data.len();
        let new_pos = match pos {
            io::SeekFrom::Start(offset) => offset as i64,
            io::SeekFrom::End(offset) => buf_len as i64 + offset,
            io::SeekFrom::Current(offset) => self.pos as i64 + offset,
        };

        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid seek to negative position",
            ));
        }

        self.pos = new_pos as usize;
        Ok(self.pos as u64)
    }
}
