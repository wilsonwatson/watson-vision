use std::{
    net::{Ipv4Addr, SocketAddrV4}, str::FromStr, sync::{Arc, Mutex}, thread::JoinHandle, time::{Duration, Instant}
};

use binrw::BinWrite;
use crossbeam_channel::{Receiver, Sender};
use nt::PublishProperties;
use once_cell::sync::Lazy;
use opencv::types::VectorOfu8;
use pipeline::{
    camera_pose_estimator::CameraPoseEstimator,
    capture::Capture,
    fiducial_detector::{self, FiducialDetector},
};
use rocket::{fairing::AdHoc, http::ContentType, response::stream::ByteStream, State};

#[macro_use]
extern crate rocket;

mod config;
pub(crate) mod nt;
pub(crate) mod types;

pub(crate) mod pipeline {
    pub mod camera_pose_estimator;
    pub mod capture;
    pub mod fiducial_detector;
}

#[get("/")]
fn index() -> (ContentType, &'static str) {
    (ContentType::HTML, include_str!("index.html"))
}

#[get("/test.mjpeg")]
async fn mjpeg_stream<'a>(
    recv: &'a State<Receiver<Vec<u8>>>,
) -> (
    ContentType,
    ByteStream<impl futures_util::Stream<Item = Vec<u8>> + 'a>,
) {
    (
        ContentType::new("multipart", "x-mixed-replace; boundary=FRAME"),
        ByteStream! {
            loop {
                yield recv.recv().unwrap();
            }
        },
    )
}

static APRILTAG_THREAD_STOP: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));
static APRILTAG_THREAD_JOINHANDLE: Lazy<Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>> =
    Lazy::new(|| Arc::new(tokio::sync::Mutex::new(None)));

static NT_TIME: Lazy<Arc<Mutex<(u32, Instant)>>> = Lazy::new(|| Arc::new(Mutex::new((0, Instant::now()))));

async fn nt_thread(data_recv: &Receiver<Vec<u8>>, config_content: &str) -> anyhow::Result<()> {
    let config: config::Config = serde_json::from_str(config_content)?;
    let server_ip = config.server_ip;
    let name = config.camera_name.clone();
    let client =
        nt::Client::try_new(SocketAddrV4::new(Ipv4Addr::from_str(&server_ip)?, 5810)).await?;
    let publisher = client
        .publish_topic(
            format!("/CameraPublisher/{}/streams", name),
            nt::Type::StringArray,
            Some(PublishProperties {
                persistent: Some(false),
                retained: Some(true),
                rest: None,
            }),
        )
        .await?;
    let my_local_ip = local_ip_address::local_ip()?.to_string();
    client
        .publish_value(
            &publisher,
            &rmpv::Value::Array(vec![rmpv::Value::String(
                format!(
                    "mjpeg:http://{}:{}/test.mjpeg",
                    my_local_ip, config.stream_port
                )
                .into(),
            )]),
        )
        .await?;
    let publisher = client
        .publish_topic(
            format!("/watson/{}", name),
            nt::Type::Raw,
            Some(PublishProperties {
                persistent: Some(false),
                retained: Some(false),
                rest: None,
            }),
        )
        .await?;
    loop {
        match APRILTAG_THREAD_STOP.lock().map(|x| *x) {
            Ok(false) => {}
            _ => break Ok(()),
        }
        *NT_TIME.lock().unwrap() = (client.server_time(), Instant::now());
        let data = data_recv.recv()?;
        let data = rmpv::Value::Binary(data);
        let fut = client.publish_value(&publisher, &data);
        tokio::select! {
            res = fut => {
                res?;
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                anyhow::bail!("Took to long. Restarting.");
            }
        }
    }
}

#[launch]
fn rocket() -> _ {
    let config_content = std::fs::read_to_string(
        std::env::args()
            .skip(1)
            .next()
            .expect("watson-vision must be called with at least one argument"),
    )
    .expect("the first argument must be a path to a config.json");
    let (send, recv) = crossbeam_channel::bounded(2);
    let (data_send, data_recv) = crossbeam_channel::bounded(0);
    let cfg2 = config_content.clone();
    *APRILTAG_THREAD_JOINHANDLE.blocking_lock() = Some(std::thread::spawn(move || {
        let config_content = cfg2;
        let data_send = data_send;
        let send: Sender<Vec<u8>> = send;
        loop {
            match APRILTAG_THREAD_STOP.lock().map(|x| *x) {
                Ok(false) => {}
                _ => break,
            }
            let config_content = config_content.clone();
            if let Err(_e) = std::panic::catch_unwind(|| {
                let config: config::Config =
                    serde_json::from_str(&config_content).unwrap();
                #[cfg(not(target_os = "linux"))]
                let mut capture = pipeline::capture::TestCapture::default();
                #[cfg(target_os = "linux")]
                let mut capture = pipeline::capture::GStreamerCapture::default();
                let mut fiducial_detector = fiducial_detector::ArucoFiducialDetector::new(
                    opencv::aruco::DICT_APRILTAG_36h11,
                );
                let mut pose_estimator =
                    pipeline::camera_pose_estimator::MultiTargetCameraPoseEstimator;
                let mut start = Instant::now();
                loop {
                    match APRILTAG_THREAD_STOP.lock().map(|x| *x) {
                        Ok(false) => {}
                        _ => break,
                    }
                    let next = Instant::now();
                    let _fps = 1.0 / next.duration_since(start).as_secs_f64();
                    start = next;
                    let (retval, mut frame) = capture.get_frame(&config);
                    if !retval {
                        std::thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    let tags = fiducial_detector.detect_fiducial(&mut frame, &config);
                    if let Some(pose) = pose_estimator.solve_camera_pose(tags, &config) {
                        let mut io = std::io::Cursor::new(Vec::with_capacity(44));
                        let (server_time, instant) = NT_TIME.lock().unwrap().to_owned();
                        let time = Instant::now().duration_since(instant).as_micros() as u32 + server_time;
                        pose.write_be_args(&mut io, (time,)).unwrap();
                        _ = data_send.send_timeout(io.into_inner(), Duration::from_millis(4));
                    }

                    let mut data = VectorOfu8::new();
                    opencv::imgcodecs::imencode_def(".jpg", &frame, &mut data).unwrap();
                    let data = (*b"--FRAME\r\nContent-Type: image/jpeg\r\n\r\n")
                        .into_iter()
                        .chain(data)
                        .chain(*b"\r\n")
                        .collect::<Vec<_>>();
                    _ = send.send_timeout(data.to_vec(), Duration::from_millis(8));
                }
            }) {
                eprintln!("Error in camera stream");
            }
        }
    }));
    let config: config::Config = serde_json::from_str(&config_content).unwrap();
    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("port", config.stream_port));
    rocket::custom(figment)
        .manage(recv)
        .attach(AdHoc::on_liftoff("Startup NetworkTables", |_| {
            Box::pin(async move {
                let data_recv = data_recv;
                tokio::spawn(async move {
                    let data_recv = data_recv;
                    let config_content = config_content.clone();
                    loop {
                        if let Err(e) = nt_thread(&data_recv, &config_content).await {
                            eprintln!("NetworkTables error: {}", e);
                            tokio::time::sleep(Duration::from_millis(500)).await;
                        }
                    }
                });
            })
        }))
        .attach(AdHoc::on_shutdown("Shutdown Apriltag Thread", |_| {
            Box::pin(async move {
                *APRILTAG_THREAD_STOP.lock().unwrap() = true;
                if let Some(handle) = APRILTAG_THREAD_JOINHANDLE.lock().await.take() {
                    handle.join().unwrap();
                }
            })
        }))
        .mount("/", routes![index, mjpeg_stream])
}
