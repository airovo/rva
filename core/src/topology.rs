use crate::model::{Scene, Topology};

pub struct Selection<'a> {
    pub topology: &'a Topology,
    /// True when no topology range matched the aspect ratio and the nearest was
    /// chosen instead.
    pub used_fallback: bool,
}

/// Select a topology for the given aspect ratio. Ranges are inclusive and the
/// first declared match wins, keeping selection deterministic.
pub fn select(scene: &Scene, aspect_ratio: f32) -> Option<Selection<'_>> {
    let mut nearest: Option<(&Topology, f32)> = None;

    for topology in &scene.topologies {
        if topology.when.contains(aspect_ratio) {
            return Some(Selection {
                topology,
                used_fallback: false,
            });
        }
        let distance = if aspect_ratio < topology.when.min() {
            topology.when.min() - aspect_ratio
        } else {
            aspect_ratio - topology.when.max()
        };
        if nearest.is_none_or(|(_, d)| distance < d) {
            nearest = Some((topology, distance));
        }
    }

    nearest.map(|(topology, _)| Selection {
        topology,
        used_fallback: true,
    })
}
