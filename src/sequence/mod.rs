//! Sequence diagrams: participants and the messages they exchange, in the order they are sent.
//!
//! The diagram runs along a time axis, down in portrait and right in landscape. Every message
//! takes a slot on that axis, fragments add a head and a foot around their messages, and the
//! participants spread evenly across the other axis. The description and the data table list
//! every message in order, so that the diagram is never the only place it is said.

mod describe;
mod draw;
mod landscape;
mod model;
mod portrait;
#[cfg(test)]
mod tests;
mod track;

pub(crate) use describe::{data_table, description};

use self::{landscape::Landscape, model::Model, portrait::Portrait};
use crate::{
    diagram,
    error::ChartWarning,
    layout::{push_title, title_extra},
    metrics::TextMetrics,
    scene::Scene,
    spec::{ChartSpec, DiagramOrientation, SequenceSpec},
};

/// The widest canvas that gets the compact layout: that of a mobile variant, whose width is at most
/// 600. A canvas that grows to fit its labels stays in the layout it was asked for.
const COMPACT_BELOW: u32 = 600;
/// Margin around the diagram.
const MARGIN: f64 = 24.0;
/// The margin, and the widest text in a participant's box, of a diagram narrower than
/// [`COMPACT_BELOW`], such as a mobile variant: there a participant's name wraps onto two lines.
const COMPACT_MARGIN: f64 = 12.0;
const COMPACT_TEXT: f64 = 64.0;
/// Space kept free below the diagram.
const BOTTOM: f64 = 16.0;
const LABEL_SIZE: f64 = 13.0;
const SUBLABEL_SIZE: f64 = 11.0;
const MESSAGE_SIZE: f64 = 12.0;
const TAG_SIZE: f64 = 11.0;
/// Distance between the baselines of a message label on two lines.
const LINE: f64 = 15.0;
/// Text inset inside a participant's box.
const PAD: f64 = 12.0;
/// The same in a diagram narrower than [`COMPACT_BELOW`], where five columns share 360 pixels.
const COMPACT_PAD: f64 = 5.0;
/// The widest message label in a compact diagram: it may reach over the lifelines beside its arrow.
const COMPACT_LABEL: f64 = 150.0;
/// Narrowest and widest box of a participant.
const MIN_BOX: f64 = 72.0;
const COMPACT_BOX: f64 = 56.0;
/// How far a numbered message's label rises so that it clears the badge on its arrow.
const BADGE_LIFT: f64 = 7.0;
const MAX_BOX: f64 = 180.0;
/// Height of the figure drawn above an actor's name.
const FIGURE: f64 = 26.0;
/// Half the width of an activation bar, and how far each nested bar moves aside.
const BAR: f64 = 5.0;
const NESTED_BAR: f64 = 4.0;
/// The loop a message to its sender draws: how far out and how far along it goes.
const LOOP_OUT: f64 = 24.0;
const LOOP_ALONG: f64 = 16.0;
/// The widest label of a message of the last participant to itself, before it wraps.
const LAST_SELF_LABEL: f64 = 170.0;
/// The widest message label in landscape, before it wraps.
const LANDSCAPE_LABEL: f64 = 160.0;
/// Growth of the slots on the time axis when the canvas leaves room.
const MAX_STRETCH: f64 = 1.5;
/// How far each nested fragment frame moves in from the one around it.
const FRAME_INSET: f64 = 5.0;
const TAG_HEIGHT: f64 = 16.0;

/// The whole diagram, laid out in whichever orientation fits.
pub(crate) fn layout(
    spec: &ChartSpec,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) -> Scene {
    let sequence = sequence(spec);
    let width = f64::from(spec.width);
    let compact = spec.width < COMPACT_BELOW;
    let margin = if compact { COMPACT_MARGIN } else { MARGIN };
    let top = 56.0 + title_extra(spec, width - 2.0 * margin, metrics);
    let model = Model::new(sequence, compact, metrics);
    let mut ignored = Vec::new();
    let landscape = match sequence.orientation {
        DiagramOrientation::Portrait => false,
        DiagramOrientation::Landscape => true,
        DiagramOrientation::Auto => {
            let portrait = Portrait::new(spec, &model, top, metrics, &mut ignored);
            let landscape = Landscape::new(spec, &model, top, metrics, &mut ignored);
            diagram::prefers_landscape(
                spec,
                (portrait.width, portrait.height),
                (landscape.width, landscape.height),
            )
        }
    };
    let mut elements = Vec::new();
    push_title(
        &mut elements,
        spec,
        margin,
        width - 2.0 * margin,
        metrics,
        warnings,
    );
    let (scene_width, scene_height) = if landscape {
        Landscape::new(spec, &model, top, metrics, warnings).draw(
            spec,
            &model,
            metrics,
            warnings,
            &mut elements,
        )
    } else {
        Portrait::new(spec, &model, top, metrics, warnings).draw(
            spec,
            &model,
            metrics,
            warnings,
            &mut elements,
        )
    };
    Scene {
        width: scene_width,
        height: scene_height,
        elements,
    }
}

/// Text inset inside a participant's box.
fn pad(compact: bool) -> f64 {
    if compact { COMPACT_PAD } else { PAD }
}

fn sequence(spec: &ChartSpec) -> &SequenceSpec {
    spec.sequence
        .as_ref()
        .expect("validated sequence diagrams carry a sequence block")
}

fn label_path(index: usize) -> String {
    format!("/sequence/messages/{index}/label")
}
