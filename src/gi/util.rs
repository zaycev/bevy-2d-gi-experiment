use bevy::asset::AssetPath;
use bevy::math::IVec2;

use crate::gi::WORKGROUP_SIZE;

pub struct AssetUtil;

impl AssetUtil
{
    pub fn gi(l: &'static str) -> AssetPath<'static>
    {
        AssetPath::parse("gi/target").with_label(l.to_owned())
    }
    pub fn camera(l: &'static str) -> AssetPath<'static>
    {
        AssetPath::parse("camera/target").with_label(l.to_owned())
    }
    pub fn material(l: &'static str) -> AssetPath<'static>
    {
        AssetPath::parse("material").with_label(l.to_owned())
    }
    pub fn mesh(l: &'static str) -> AssetPath<'static>
    {
        AssetPath::parse("mesh").with_label(l.to_owned())
    }
}

pub fn align_to_work_group_grid(size: IVec2) -> IVec2
{
    let wg_size = WORKGROUP_SIZE as i32;
    // Only add padding if necessary
    IVec2::new(
        if size.x % wg_size == 0 {
            size.x
        } else {
            size.x + wg_size - size.x % wg_size
        },
        if size.y % wg_size == 0 {
            size.y
        } else {
            size.y + wg_size - size.y % wg_size
        },
    )
}
