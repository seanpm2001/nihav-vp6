//! Decoder interface definitions.
pub use crate::frame::*;
use crate::io::byteio::ByteIOError;
use crate::io::bitreader::BitReaderError;
use crate::io::codebook::CodebookError;
pub use crate::options::*;
pub use std::str::FromStr;

/// A list specifying general decoding errors.
#[derive(Debug,Clone,Copy,PartialEq)]
#[allow(dead_code)]
pub enum DecoderError {
    /// No frame was provided.
    NoFrame,
    /// Allocation failed.
    AllocError,
    /// Operation requires repeating.
    TryAgain,
    /// Invalid input data was provided.
    InvalidData,
    /// Provided input turned out to be incomplete.
    ShortData,
    /// Decoder could not decode provided frame because it references some missing previous frame.
    MissingReference,
    /// Feature is not implemented.
    NotImplemented,
    /// Some bug in decoder. It should not happen yet it might.
    Bug,
}

/// A specialised `Result` type for decoding operations.
pub type DecoderResult<T> = Result<T, DecoderError>;

impl From<ByteIOError> for DecoderError {
    fn from(_: ByteIOError) -> Self { DecoderError::ShortData }
}

impl From<BitReaderError> for DecoderError {
    fn from(e: BitReaderError) -> Self {
        match e {
            BitReaderError::BitstreamEnd => DecoderError::ShortData,
            _ => DecoderError::InvalidData,
        }
    }
}

impl From<CodebookError> for DecoderError {
    fn from(_: CodebookError) -> Self { DecoderError::InvalidData }
}

impl From<AllocatorError> for DecoderError {
    fn from(_: AllocatorError) -> Self { DecoderError::AllocError }
}

/// Auxiliary structure for storing data used by decoder but also controlled by the caller.
pub struct NADecoderSupport {
    /// Frame buffer pool for 8-bit or packed video frames.
    pub pool_u8:        NAVideoBufferPool<u8>,
    /// Frame buffer pool for 16-bit video frames.
    pub pool_u16:       NAVideoBufferPool<u16>,
    /// Frame buffer pool for 32-bit video frames.
    pub pool_u32:       NAVideoBufferPool<u32>,
}

impl NADecoderSupport {
    /// Constructs a new instance of `NADecoderSupport`.
    pub fn new() -> Self {
        Self {
            pool_u8:        NAVideoBufferPool::new(0),
            pool_u16:       NAVideoBufferPool::new(0),
            pool_u32:       NAVideoBufferPool::new(0),
        }
    }
}

impl Default for NADecoderSupport {
    fn default() -> Self { Self::new() }
}

/// Decoder trait.
pub trait NADecoder: NAOptionHandler {
    /// Initialises the decoder.
    ///
    /// It takes [`NADecoderSupport`] allocated by the caller and `NACodecInfoRef` provided by demuxer.
    ///
    /// [`NADecoderSupport`]: ./struct.NADecoderSupport.html
    fn init(&mut self, supp: &mut NADecoderSupport, info: NACodecInfoRef) -> DecoderResult<()>;
    /// Decodes a single frame.
    fn decode(&mut self, supp: &mut NADecoderSupport, pkt: &NAPacket) -> DecoderResult<NAFrameRef>;
    /// Tells decoder to clear internal state (e.g. after error or seeking).
    fn flush(&mut self);
}

/// Decoder information used during creating a decoder for requested codec.
#[derive(Clone,Copy)]
pub struct DecoderInfo {
    /// Short decoder name.
    pub name: &'static str,
    /// The function that creates a decoder instance.
    pub get_decoder: fn () -> Box<dyn NADecoder + Send>,
}

/// Structure for registering known decoders.
///
/// It is supposed to be filled using `register_all_decoders()` from some decoders crate and then it can be used to create decoders for the requested codecs.
#[derive(Default)]
pub struct RegisteredDecoders {
    decs:   Vec<DecoderInfo>,
}

impl RegisteredDecoders {
    /// Constructs a new instance of `RegisteredDecoders`.
    pub fn new() -> Self {
        Self { decs: Vec::new() }
    }
    /// Adds another decoder to the registry.
    pub fn add_decoder(&mut self, dec: DecoderInfo) {
        self.decs.push(dec);
    }
    /// Searches for the decoder for the provided name and returns a function for creating it on success.
    pub fn find_decoder(&self, name: &str) -> Option<fn () -> Box<dyn NADecoder + Send>> {
        for &dec in self.decs.iter() {
            if dec.name == name {
                return Some(dec.get_decoder);
            }
        }
        None
    }
    /// Provides an iterator over currently registered decoders.
    pub fn iter(&self) -> std::slice::Iter<DecoderInfo> {
        self.decs.iter()
    }
}

/// Frame skipping mode for decoders.
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum FrameSkipMode {
    /// Decode all frames.
    None,
    /// Decode all key frames.
    KeyframesOnly,
    /// Decode only intra frames.
    IntraOnly,
}

impl Default for FrameSkipMode {
    fn default() -> Self {
        FrameSkipMode::None
    }
}

impl FromStr for FrameSkipMode {
    type Err = DecoderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            FRAME_SKIP_OPTION_VAL_NONE      => Ok(FrameSkipMode::None),
            FRAME_SKIP_OPTION_VAL_KEYFRAME  => Ok(FrameSkipMode::KeyframesOnly),
            FRAME_SKIP_OPTION_VAL_INTRA     => Ok(FrameSkipMode::IntraOnly),
            _ => Err(DecoderError::InvalidData),
        }
    }
}

impl ToString for FrameSkipMode {
    fn to_string(&self) -> String {
        match *self {
            FrameSkipMode::None             => FRAME_SKIP_OPTION_VAL_NONE.to_string(),
            FrameSkipMode::KeyframesOnly    => FRAME_SKIP_OPTION_VAL_KEYFRAME.to_string(),
            FrameSkipMode::IntraOnly        => FRAME_SKIP_OPTION_VAL_INTRA.to_string(),
        }
    }
}

/// A list specifying general encoding errors.
#[derive(Debug,Clone,Copy,PartialEq)]
#[allow(dead_code)]
pub enum EncoderError {
    /// No frame was provided.
    NoFrame,
    /// Allocation failed.
    AllocError,
    /// Operation requires repeating.
    TryAgain,
    /// Input format is not supported by codec.
    FormatError,
    /// Invalid input parameters were provided.
    InvalidParameters,
    /// Feature is not implemented.
    NotImplemented,
    /// Some bug in encoder. It should not happen yet it might.
    Bug,
}

/// A specialised `Result` type for decoding operations.
pub type EncoderResult<T> = Result<T, EncoderError>;

impl From<ByteIOError> for EncoderError {
    fn from(_: ByteIOError) -> Self { EncoderError::Bug }
}

impl From<AllocatorError> for EncoderError {
    fn from(_: AllocatorError) -> Self { EncoderError::AllocError }
}

/// Encoding parameter flag to force constant bitrate mode.
pub const ENC_MODE_CBR: u64 = 1 << 0;
/// Encoding parameter flag to force constant framerate mode.
pub const ENC_MODE_CFR: u64 = 1 << 1;

/// Encoding parameters.
#[derive(Clone,Copy,PartialEq)]
pub struct EncodeParameters {
    /// Input format.
    pub format:     NACodecTypeInfo,
    /// Time base numerator. Ignored for audio.
    pub tb_num:     u32,
    /// Time base denominator. Ignored for audio.
    pub tb_den:     u32,
    /// Bitrate in kilobits per second.
    pub bitrate:    u32,
    /// A collection of various boolean encoder settings like CBR mode.
    ///
    /// See `ENC_MODE_*` constants for available options.
    pub flags:      u64,
    /// Encoding quality.
    pub quality:    u8,
}

impl Default for EncodeParameters {
    fn default() -> EncodeParameters {
        EncodeParameters {
            format:     NACodecTypeInfo::None,
            tb_num:     0,
            tb_den:     0,
            bitrate:    0,
            flags:      0,
            quality:    0,
        }
    }
}

/// Encoder trait.
///
/// Overall encoding is more complex than decoding.
/// There are at least two issues that should be addressed: input format and the need for lookahead.
///
/// Some formats (like MPEG-1 ones) have fixed picture dimensions and framerate, or sampling rate.
/// Some formats accept only pictures with dimensions being multiple of eight or sixteen.
/// Some audio formats work only with monaural sound.
/// In order to account for all this user first needs to check whether encoder can handle provided input format as is or some conversion is required.
/// That is why `NAEncoder` has [`negotiate_format`] function that performs such check and returns what encoder can handle.
///
/// Additionally, encoders for complex formats often need several frames lookahead to encode data efficiently, actual frame encoding may take place only when some frames are accumulated.
/// That is why encoder has two functions, one for queueing frames for encoding and one for obtaining encoded packets when they are available.
/// In result encoder should first queue a frame for encoding with [`encode`] and then retrieve zero or more encoded packets with [`get_packet`] in a loop.
///
/// Overall encoding loop should look like this:
/// ```ignore
/// let encoder = ...; // create encoder
/// let enc_params = encoder.negotiate_format(input_enc_params)?; // negotiate format
/// let output_stream = encoder.init(stream_no, enc_params)?;
/// while let Some(frame) = queue.get_frame() {
///     // convert to the format encoder expects if required
///     encoder.encode(frame)?;
///     while let Some(enc_pkt) = encoder.get_packet()? {
///         // send encoded packet to a muxer for example
///     }
/// }
/// // retrieve the rest of encoded packets
/// encoder.flush()?;
/// while let Ok(enc_pkt) = encoder.get_packet()? {
///     // send encoded packet to a muxer for example
/// }
/// ```
///
/// [`negotiate_format`]: ./trait.NAEncoder.html#tymethod.negotiate_format
/// [`encode`]: ./trait.NAEncoder.html#tymethod.encode
/// [`get_packet`]: ./trait.NAEncoder.html#tymethod.get_packet
pub trait NAEncoder: NAOptionHandler {
    /// Tries to negotiate input format.
    ///
    /// This function takes input encoding parameters and returns adjusted encoding parameters if input ones make sense.
    /// If input parameters are empty then the default parameters are returned.
    ///
    /// # Example
    /// ```ignore
    /// let enc_params = [ EncodeParameters {...}, ..., EncodeParameters::default() ];
    /// let mut target_params = EncodeParameters::default();
    /// for params in enc_params.iter() {
    ///     if let Ok(dparams) = encoder.negotiate_format(params) {
    ///         target_params = dparams;
    ///         break;
    ///     }
    /// }
    /// // since negotiate_format(EncodeParameters::default()) will return a valid format, target_params should be valid here
    /// let stream = encoder.init(stream_id, target_params)?;
    /// // convert input into format defined in target_params, feed to the encoder, ...
    /// ```
    fn negotiate_format(&self, encinfo: &EncodeParameters) -> EncoderResult<EncodeParameters>;
    /// Initialises the encoder.
    fn init(&mut self, stream_id: u32, encinfo: EncodeParameters) -> EncoderResult<NAStreamRef>;
    /// Takes a single frame for encoding.
    fn encode(&mut self, frm: &NAFrame) -> EncoderResult<()>;
    /// Returns encoded packet if available.
    fn get_packet(&mut self) -> EncoderResult<Option<NAPacket>>;
    /// Tells encoder to encode all data it currently has.
    fn flush(&mut self) -> EncoderResult<()>;
}

/// Encoder information used during creating an encoder for requested codec.
#[derive(Clone,Copy)]
pub struct EncoderInfo {
    /// Short encoder name.
    pub name: &'static str,
    /// The function that creates an encoder instance.
    pub get_encoder: fn () -> Box<dyn NAEncoder + Send>,
}

/// Structure for registering known encoders.
///
/// It is supposed to be filled using `register_all_decoders()` from some encoders crate and then it can be used to create encoders for the requested codecs.
#[derive(Default)]
pub struct RegisteredEncoders {
    encs:   Vec<EncoderInfo>,
}

impl RegisteredEncoders {
    /// Constructs a new instance of `RegisteredEncoders`.
    pub fn new() -> Self {
        Self { encs: Vec::new() }
    }
    /// Adds another encoder to the registry.
    pub fn add_encoder(&mut self, enc: EncoderInfo) {
        self.encs.push(enc);
    }
    /// Searches for the encoder for the provided name and returns a function for creating it on success.
    pub fn find_encoder(&self, name: &str) -> Option<fn () -> Box<dyn NAEncoder + Send>> {
        for &enc in self.encs.iter() {
            if enc.name == name {
                return Some(enc.get_encoder);
            }
        }
        None
    }
    /// Provides an iterator over currently registered encoders.
    pub fn iter(&self) -> std::slice::Iter<EncoderInfo> {
        self.encs.iter()
    }
}

