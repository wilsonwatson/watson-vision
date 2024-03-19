use opencv::{types::VectorOfu8, videoio::VideoCaptureTrait};

use crate::config::Config;

pub trait Capture: Default {
    fn get_frame(&mut self, config_store: &Config) -> (bool, opencv::prelude::Mat);

    fn config_changed(config_a: Option<&Config>, config_b: Option<&Config>) -> bool {
        if let Some(config_a) = config_a {
            if let Some(config_b) = config_b {
                return config_a.video_path != config_b.video_path
                    || config_a.width != config_b.width
                    || config_a.height != config_b.height
                    || config_a.exposure != config_b.exposure
                    || config_a.auto_exposure != config_b.auto_exposure
                    || config_a.gain != config_b.gain;
            } else {
                return true;
            }
        } else if let Some(_config_b) = config_b {
            return true;
        } else {
            return false;
        }
    }
}

#[derive(Debug, Default)]
pub struct DefaultCapture {
    video: Option<opencv::videoio::VideoCapture>,
    last_config: Option<Config>,
}

impl Capture for DefaultCapture {
    fn get_frame(&mut self, config_store: &Config) -> (bool, opencv::prelude::Mat) {
        if Self::config_changed(self.last_config.as_ref(), Some(&config_store)) {
            if let Some(mut video) = self.video.take() {
                video.release().unwrap();
            }
        }
        if let None = self.video {
            let mut video =
                opencv::videoio::VideoCapture::new(0, opencv::videoio::CAP_V4L).unwrap();
            video
                .set(
                    opencv::videoio::CAP_PROP_FRAME_WIDTH,
                    config_store.width as f64,
                )
                .unwrap();
            video
                .set(
                    opencv::videoio::CAP_PROP_FRAME_HEIGHT,
                    config_store.height as f64,
                )
                .unwrap();
            video
                .set(
                    opencv::videoio::CAP_PROP_AUTO_EXPOSURE,
                    config_store.auto_exposure as f64,
                )
                .unwrap();
            video
                .set(
                    opencv::videoio::CAP_PROP_EXPOSURE,
                    config_store.exposure as f64,
                )
                .unwrap();
            video
                .set(opencv::videoio::CAP_PROP_GAIN, config_store.gain as f64)
                .unwrap();
            self.video = Some(video);
        }
        self.last_config = Some(config_store.clone());

        let mut mat = opencv::prelude::Mat::default();
        let ret = self.video.as_mut().unwrap().read(&mut mat).unwrap();
        (ret, mat)
    }
}

#[derive(Debug, Default)]
pub struct GStreamerCapture {
    video: Option<opencv::videoio::VideoCapture>,
    last_config: Option<Config>,
}

impl Capture for GStreamerCapture {
    fn get_frame(&mut self, config_store: &Config) -> (bool, opencv::prelude::Mat) {
        if Self::config_changed(self.last_config.as_ref(), Some(&config_store)) {
            if let Some(mut video) = self.video.take() {
                video.release().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
        if let None = self.video {
            if config_store.video_path == "" {
                println!("No camera ID, waiting to start capture session.");
            } else {
                println!("Starting capture session");
                self.video = Some(opencv::videoio::VideoCapture::from_file(&format!("v4l2src device={} extra_controls=\"c,exposure_auto={},exposure_absolute={},gain={},sharpness=0,brightness=0\" ! image/jpeg,format=MJPG,width={},height={} ! jpegdec ! video/x-raw ! appsink drop=1", config_store.video_path, config_store.auto_exposure, config_store.exposure, config_store.gain, config_store.width, config_store.height), opencv::videoio::CAP_GSTREAMER).unwrap());
                println!("Capture session ready");
            }
        }

        self.last_config = Some(config_store.clone());

        let mut image = opencv::prelude::Mat::default();
        if let Some(video) = self.video.as_mut() {
            let retval = video.read(&mut image).unwrap();
            if !retval {
                video.release().expect("Capture session failed, restarting");
                panic!("Capture session failed, restarting");
            }
            (retval, image)
        } else {
            (false, image)
        }
    }
}

#[derive(Debug)]
pub struct TestCapture {
    test_image: opencv::core::Mat,
}

impl Default for TestCapture {
    fn default() -> Self {
        let data = VectorOfu8::from_slice(include_bytes!("test.png"));
        let test_image = opencv::imgcodecs::imdecode(&data, opencv::imgcodecs::IMREAD_COLOR).unwrap();
        Self { test_image }
    }
}

impl Capture for TestCapture {
    fn get_frame(&mut self, _config_store: &Config) -> (bool, opencv::prelude::Mat) {
        (true, self.test_image.clone())
    }
}

