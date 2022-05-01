use bevy::prelude::*;

use crate::{animations::*, player::*};

pub(crate) fn render_billboards(
    mut q: ParamSet<(
        Query<(&Parent, &mut Transform), With<Billboard>>,
        Query<(&GlobalTransform, &Transform), With<PlayerPerspective>>,
    )>,
    parent_query: Query<(&Player, &GlobalTransform)>,
) {
    let viewer_transform = q.p1().iter().last().unwrap().1.translation;
    for (parent, mut t) in q.p0().iter_mut() {
        let parent_position = parent_query.get(parent.0).unwrap().1.translation;
        let parent_rotation = parent_query.get(parent.0).unwrap().0.rotation;

        let delta_z = parent_position.z - viewer_transform.z;
        let delta_x = parent_position.x - viewer_transform.x;
        let angle = delta_x.atan2(delta_z);

        let rot_y = Quat::from_rotation_y(std::f32::consts::PI + angle - parent_rotation);

        t.rotation = rot_y;
    }
}
