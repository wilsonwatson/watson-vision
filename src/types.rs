use binrw::BinWrite;
use nalgebra::{Isometry3, Vector3};
use opencv::core::VecN;


#[derive(Debug)]
pub struct FiducialImageObservation {
    pub tag_id: u64,
    pub corners: [[f64; 2]; 4],
}

#[derive(Debug)]
pub struct FiducialPoseObservation {
    pub tag_id: u64,
    pub pose_0: Isometry3<f64>,
    pub error_0: f64,
    pub pose_1: Isometry3<f64>,
    pub error_1: f64,
}

#[derive(Debug)]
pub struct CameraPoseObservation {
    pub tag_ids: Vec<u64>,
    pub pose_0: Isometry3<f64>,
    pub error_0: f64,
    pub pose_1: Option<Isometry3<f64>>,
    pub error_1: Option<f64>,
}

impl BinWrite for CameraPoseObservation {
    type Args<'a> = (u32,);

    fn write_options<W: std::io::prelude::Write + std::io::prelude::Seek>(
        &self,
        writer: &mut W,
        _endian: binrw::Endian,
        (time,): Self::Args<'_>,
    ) -> binrw::prelude::BinResult<()> {
        time.write_be(writer)?;
        (self.tag_ids.len() as i32).write_be(writer)?;
        for tag in &self.tag_ids {
            (*tag as i32).write_be(writer)?;
        }
        if let Some(_) = self.pose_1.as_ref() {
            1u8.write_be(writer)?;
        } else {
            0u8.write_be(writer)?;
        }

        self.pose_0.translation.x.write_be(writer)?;
        self.pose_0.translation.y.write_be(writer)?;
        self.pose_0.translation.z.write_be(writer)?;
        self.pose_0.rotation.w.write_be(writer)?;
        self.pose_0.rotation.vector().x.write_be(writer)?;
        self.pose_0.rotation.vector().y.write_be(writer)?;
        self.pose_0.rotation.vector().z.write_be(writer)?;
        self.error_0.write_be(writer)?;
        
        if let Some(pose_1) = self.pose_1.as_ref() {
            pose_1.translation.x.write_be(writer)?;
            pose_1.translation.y.write_be(writer)?;
            pose_1.translation.z.write_be(writer)?;
            pose_1.rotation.w.write_be(writer)?;
            pose_1.rotation.vector().x.write_be(writer)?;
            pose_1.rotation.vector().y.write_be(writer)?;
            pose_1.rotation.vector().z.write_be(writer)?;
            self.error_1.as_ref().unwrap().write_be(writer)?;
        }
        
        Ok(())
    }
}

pub fn isometry_from_opencv(t: VecN<f64, 3>, r: VecN<f64, 3>) -> Isometry3<f64> {
    Isometry3::new(Vector3::new(t.0[2], -t.0[0], -t.0[1]), Vector3::new(r.0[2], -r.0[0], -r.0[1]))
}

pub fn translation_to_opencv(translation: Vector3<f64>) -> VecN<f64, 3> {
    return VecN::from_array([-translation.y, -translation.z, translation.x])
}