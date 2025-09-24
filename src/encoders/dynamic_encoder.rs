use crossbeam::channel::Receiver;
use ffmpeg_next::codec::encoder;

use crate::{
    encoders::{
        vaapi_encoder::VaapiEncoder,
        video::{PipewireSPA, ProcessingThread},
    },
    types::{
        config::VideoEncoder as VideoEncoderType,
        error::{Result, WaycapError},
        video_frame::{EncodedVideoFrame, RawVideoFrame},
    },
    waycap_egl::{EglContext, GpuVendor},
    VideoEncoder,
};

#[cfg(feature = "nvenc")]
use crate::encoders::nvenc_encoder::NvencEncoder;

pub enum DynamicEncoder {
    Vaapi(VaapiEncoder),
    #[cfg(feature = "nvenc")]
    Nvenc(NvencEncoder),
}

impl DynamicEncoder {
    pub(crate) fn new(
        encoder_type: Option<VideoEncoderType>,
        width: u32,
        height: u32,
        quality_preset: crate::types::config::QualityPreset,
    ) -> crate::types::error::Result<DynamicEncoder> {
        let encoder_type = match encoder_type {
            Some(typ) => typ,
            None => {
                // Dummy dimensions we just use this go get GPU vendor then drop it
                let dummy_context = EglContext::new(100, 100)?;
                match dummy_context.get_gpu_vendor() {
                    GpuVendor::NVIDIA => {
                        cfg_if::cfg_if! {
                            if #[cfg(feature = "nvenc")] {
                                VideoEncoderType::H264Nvenc
                            } else {
                                VideoEncoderType::H264Vaapi
                            }
                        }
                    },
                    GpuVendor::AMD | GpuVendor::INTEL => VideoEncoderType::H264Vaapi,
                    GpuVendor::UNKNOWN => {
                        return Err(WaycapError::Init(
                            "Unknown/Unimplemented GPU vendor".to_string(),
                        ));
                    }
                }
            }
        };
        Ok(match encoder_type {
            #[cfg(feature = "nvenc")]
            VideoEncoderType::H264Nvenc => {
                DynamicEncoder::Nvenc(NvencEncoder::new(width, height, quality_preset)?)
            }
            VideoEncoderType::H264Vaapi => {
                DynamicEncoder::Vaapi(VaapiEncoder::new(width, height, quality_preset)?)
            }
        })
    }
}

impl VideoEncoder for DynamicEncoder {
    type Output = EncodedVideoFrame;

    fn reset(&mut self) -> Result<()> {
        match self {
            DynamicEncoder::Vaapi(enc) => enc.reset(),
            #[cfg(feature = "nvenc")]
            DynamicEncoder::Nvenc(enc) => enc.reset(),
        }
    }

    fn output(&mut self) -> Option<Receiver<Self::Output>> {
        match self {
            DynamicEncoder::Vaapi(enc) => enc.output(),
            #[cfg(feature = "nvenc")]
            DynamicEncoder::Nvenc(enc) => enc.output(),
        }
    }

    fn drop_processor(&mut self) {
        match self {
            DynamicEncoder::Vaapi(enc) => enc.drop_processor(),
            #[cfg(feature = "nvenc")]
            DynamicEncoder::Nvenc(enc) => enc.drop_processor(),
        }
    }

    fn drain(&mut self) -> Result<()> {
        match self {
            DynamicEncoder::Vaapi(enc) => enc.drain(),
            #[cfg(feature = "nvenc")]
            DynamicEncoder::Nvenc(enc) => enc.drain(),
        }
    }

    fn get_encoder(&self) -> &Option<encoder::Video> {
        match self {
            DynamicEncoder::Vaapi(enc) => enc.get_encoder(),
            #[cfg(feature = "nvenc")]
            DynamicEncoder::Nvenc(enc) => enc.get_encoder(),
        }
    }
}

impl ProcessingThread for DynamicEncoder {
    fn process(&mut self, frame: RawVideoFrame) -> Result<()> {
        match self {
            DynamicEncoder::Vaapi(enc) => enc.process(frame),
            #[cfg(feature = "nvenc")]
            DynamicEncoder::Nvenc(enc) => enc.process(frame),
        }
    }
    fn thread_setup(&mut self) -> Result<()> {
        match self {
            DynamicEncoder::Vaapi(enc) => enc.thread_setup(),
            #[cfg(feature = "nvenc")]
            DynamicEncoder::Nvenc(enc) => enc.thread_setup(),
        }
    }

    fn thread_teardown(&mut self) -> Result<()> {
        match self {
            DynamicEncoder::Vaapi(enc) => enc.thread_teardown(),
            #[cfg(feature = "nvenc")]
            DynamicEncoder::Nvenc(enc) => enc.thread_teardown(),
        }
    }
}

impl PipewireSPA for DynamicEncoder {
    fn get_spa_definition() -> Result<pipewire::spa::pod::Object> {
        let dummy_context = EglContext::new(100, 100)?;
        match dummy_context.get_gpu_vendor() {
            GpuVendor::NVIDIA => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "nvenc")] {
                        NvencEncoder::get_spa_definition()
                    } else {
                        VaapiEncoder::get_spa_definition()
                    }
                }
            },
            GpuVendor::AMD | GpuVendor::INTEL => VaapiEncoder::get_spa_definition(),
            GpuVendor::UNKNOWN => Err(WaycapError::Init(
                "Unknown/Unimplemented GPU vendor".to_string(),
            )),
        }
    }
}
