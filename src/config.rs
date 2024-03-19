use nalgebra::{Isometry3, Quaternion, Translation3, UnitQuaternion};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub video_path: String,
    pub width: u32,
    pub height: u32,
    pub auto_exposure: u32,
    pub exposure: u32,
    pub gain: u32,
    pub fiducial_size_m: f64,

    pub server_ip: String,
    pub camera_name: String,
    pub stream_port: u64,
    pub has_calibration: bool,
    #[serde(deserialize_with = "deserialize_mat3")]
    pub camera_matrix: opencv::core::Mat,
    #[serde(deserialize_with = "deserialize_vecn")]
    pub distortion_coefficients: opencv::core::Mat,
    pub tag_layout: TagLayout,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TagLayout {
    pub tags: Vec<Tag>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Tag {
    #[serde(rename = "ID")]
    pub id: u64,
    #[serde(deserialize_with = "deserialize_isometry3")]
    pub pose: Isometry3<f64>,
}

fn deserialize_isometry3<'de, D>(d: D) -> Result<Isometry3<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct V3 {
        x: f64,
        y: f64,
        z: f64,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    struct V4 {
        x: f64,
        y: f64,
        z: f64,
        w: f64,
    }
    #[derive(Deserialize)]
    struct Rotation {
        quaternion: V4,
    }
    #[derive(Deserialize)]
    struct Pose {
        translation: V3,
        rotation: Rotation,
    }

    let p = Pose::deserialize(d)?;
    let iso = Isometry3::from_parts(
        Translation3::new(p.translation.x, p.translation.y, p.translation.z),
        UnitQuaternion::from_quaternion(Quaternion::new(
            p.rotation.quaternion.w,
            p.rotation.quaternion.x,
            p.rotation.quaternion.y,
            p.rotation.quaternion.z,
        )),
    );
    Ok(iso)
}

fn deserialize_mat3<'de, D>(d: D) -> Result<opencv::core::Mat, D::Error>
where
    D: Deserializer<'de>,
{
    let res = <[f64; 9]>::deserialize(d)?;
    Ok(opencv::core::Mat::from_slice_rows_cols(&res, 3, 3).unwrap())
}

fn deserialize_vecn<'de, D>(d: D) -> Result<opencv::core::Mat, D::Error>
where
    D: Deserializer<'de>,
{
    let res = <Vec<f64>>::deserialize(d)?;
    Ok(opencv::core::Mat::from_slice(&res).unwrap())
}
