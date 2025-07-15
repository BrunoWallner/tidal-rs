use reqwest::{Client, Response};
use std::io;

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
    seekable: bool,
}

impl Stream {
    pub async fn init<T: Streamable>(t: T, buffer_size: usize, seekable: bool) -> io::Result<Self> {
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
            seekable,
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

        // delete unused
        if !self.seekable {
            self.data.drain(0..self.pos);
            self.pos = 0;
        }

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

use symphonia::{
    core::{
        audio::SampleBuffer,
        codecs::{CODEC_TYPE_NULL, Decoder},
        formats::{FormatOptions, FormatReader, SeekMode, SeekTo},
        io::{MediaSource, MediaSourceStream},
        meta::MetadataOptions,
        probe::Hint,
        units::Time,
    },
    default::{get_codecs, get_probe},
};

impl MediaSource for Stream {
    fn is_seekable(&self) -> bool {
        self.seekable
    }

    fn byte_len(&self) -> Option<u64> {
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("backend error")]
    Backend,
    #[error("no track found")]
    NoTrack,
}
impl From<symphonia::core::errors::Error> for DecodeError {
    fn from(_: symphonia::core::errors::Error) -> Self {
        Self::Backend
    }
}

pub struct AudioDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    data: Vec<f32>,
    pos: usize,
}
impl AudioDecoder {
    pub fn from_stream(stream: Stream) -> Result<Self, DecodeError> {
        let mss = MediaSourceStream::new(Box::new(stream), Default::default());

        let hint = Hint::new();

        let probed = get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;
        let format_reader = probed.format;
        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or(DecodeError::NoTrack)?;
        let track_id = track.id;

        let decoder = get_codecs().make(
            &track.codec_params,
            &symphonia::core::codecs::DecoderOptions::default(),
        )?;

        let data = Vec::new();
        let pos = 0;

        Ok(Self {
            format_reader,
            decoder,
            track_id,
            data,
            pos,
        })
    }

    fn fetch_samples(&mut self) -> usize {
        let Ok(packet) = self.format_reader.next_packet() else {
            return 0;
        };
        if packet.track_id() != self.track_id {
            return 0;
        }
        let Ok(decoded) = self.decoder.decode(&packet) else {
            return 0;
        };
        let mut buffer = SampleBuffer::<f32>::new(decoded.frames() as u64, *decoded.spec());
        buffer.copy_interleaved_ref(decoded);
        let samples = buffer.samples();
        self.data.extend_from_slice(samples);

        samples.len()
    }

    pub fn seek(&mut self, seconds: u64) {
        let _ = self.format_reader.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time: Time::new(seconds, 0.0),
                track_id: None,
            },
        );
    }

    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let mut read = 0;
        let output_size = output.len();

        let to_read = (self.pos + output_size).saturating_sub(self.data.len());
        while read < to_read {
            let samples_read = self.fetch_samples();
            if samples_read == 0 {
                break;
            }
            read += samples_read;
        }
        let available = self.data.len().saturating_sub(self.pos);
        let to_copy = output_size.min(available);
        let copy_start = self.pos;
        let copy_end = copy_start + to_copy;
        output[..to_copy].copy_from_slice(&self.data[copy_start..copy_end]);
        self.pos += to_copy;

        self.data.drain(0..self.pos);
        self.pos = 0;

        to_copy
    }
}
