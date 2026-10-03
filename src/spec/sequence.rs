use serde::{Deserialize, Serialize};

use super::{ChartSpec, is_false, validate_text};
use crate::error::{ChartError, ChartWarning};

/// Participants of one sequence diagram; beyond this the columns get too narrow to label.
pub(crate) const MAX_PARTICIPANTS: usize = 12;
/// Messages of one sequence diagram; beyond this a diagram stops being readable at a glance.
pub(crate) const MAX_MESSAGES: usize = 60;
/// Fragments of one sequence diagram.
pub(crate) const MAX_FRAGMENTS: usize = 12;
/// How deeply fragments may nest inside each other.
pub(crate) const MAX_FRAGMENT_DEPTH: usize = 3;

/// A sequence diagram: participants side by side and the messages they exchange, in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SequenceSpec {
    pub participants: Vec<ParticipantSpec>,
    /// The messages in the order they are sent.
    pub messages: Vec<MessageSpec>,
    /// Frames around a run of consecutive messages, such as an alternative or a loop.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fragments: Vec<FragmentSpec>,
    /// Writes the number of each message in front of its label.
    #[serde(default, skip_serializing_if = "is_false")]
    pub numbered: bool,
    /// Which way the diagram runs; see [`DiagramOrientation`].
    #[serde(default, skip_serializing_if = "DiagramOrientation::is_auto")]
    pub orientation: DiagramOrientation,
}

/// Which way a diagram runs on its canvas.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagramOrientation {
    /// Portrait where the diagram fits the canvas that way, otherwise landscape where that fits,
    /// otherwise portrait. A mobile variant decides again at its own size.
    #[default]
    Auto,
    /// For tall formats: participants side by side, time running down.
    Portrait,
    /// For wide formats: participants one below the other, time running right.
    Landscape,
}

impl DiagramOrientation {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub(super) const fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParticipantSpec {
    /// What messages refer to the participant by: a letter, then letters, digits, `-` or `_`.
    pub id: String,
    pub label: String,
    /// A second, smaller line under the label, such as a technology.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    #[serde(default, skip_serializing_if = "ParticipantKind::is_service")]
    pub kind: ParticipantKind,
}

/// What a participant is; each kind has its own shape, so that it does not rest on color.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantKind {
    /// A person or a role, drawn as a figure.
    Actor,
    /// A system or component, drawn as a box.
    #[default]
    Service,
    /// A data store, drawn as a cylinder.
    Database,
    /// A queue or a topic, drawn as a box with a stack behind it.
    Queue,
    /// A system outside the diagram's scope, drawn as a dashed box.
    External,
}

impl ParticipantKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_service(&self) -> bool {
        matches!(self, Self::Service)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MessageSpec {
    /// The `id` of the sender.
    pub from: String,
    /// The `id` of the receiver; the sender itself for a message to itself.
    pub to: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "MessageKind::is_call")]
    pub kind: MessageKind,
}

/// How a message is sent. Each kind has its own line and arrowhead.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageKind {
    /// A call that waits for its reply: a solid line, a filled head. It activates the receiver.
    #[default]
    Call,
    /// The reply to a call: a dashed line, an open head. It ends the sender's last activation.
    Reply,
    /// A message that does not wait: a solid line, an open head.
    Async,
}

impl MessageKind {
    // serde hands this function a reference, so the signature follows serde's shape.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn is_call(&self) -> bool {
        matches!(self, Self::Call)
    }
}

/// A frame around the messages `from` to `to`, both counted from 0 and included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FragmentSpec {
    pub kind: FragmentKind,
    /// The condition or name the frame shows after its kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub from: usize,
    pub to: usize,
    /// Further branches of an `alt` fragment, each starting at a message inside the frame.
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "else")]
    pub branches: Vec<BranchSpec>,
}

/// What a fragment frames.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FragmentKind {
    /// One of several branches happens.
    Alt,
    /// The messages happen only under a condition.
    Opt,
    /// The messages repeat.
    Loop,
    /// The messages happen in parallel.
    Par,
    /// The messages must not be interrupted.
    Critical,
    /// The messages end the enclosing interaction.
    Break,
}

impl FragmentKind {
    /// The keyword the frame shows in its corner.
    pub(crate) const fn keyword(self) -> &'static str {
        match self {
            Self::Alt => "alt",
            Self::Opt => "opt",
            Self::Loop => "loop",
            Self::Par => "par",
            Self::Critical => "critical",
            Self::Break => "break",
        }
    }
}

/// A further branch of an `alt` fragment, from the message `from` to the next branch or the end
/// of the frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BranchSpec {
    pub from: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl SequenceSpec {
    /// The index of the participant with `id`.
    pub(crate) fn participant(&self, id: &str) -> Option<usize> {
        self.participants
            .iter()
            .position(|participant| participant.id == id)
    }

    /// Sender and receiver of every message, as participant indices.
    pub(crate) fn ends(&self) -> Vec<(usize, usize)> {
        self.messages
            .iter()
            .map(|message| {
                (
                    self.participant(&message.from)
                        .expect("validated messages name participants"),
                    self.participant(&message.to)
                        .expect("validated messages name participants"),
                )
            })
            .collect()
    }

    /// How many fragments enclose fragment `index`.
    pub(crate) fn depth(&self, index: usize) -> usize {
        let fragment = &self.fragments[index];
        self.fragments
            .iter()
            .enumerate()
            .filter(|(other, outer)| {
                *other != index
                    && encloses(outer, fragment)
                    && !(encloses(fragment, outer) && *other > index)
            })
            .count()
    }
}

/// Whether `outer` spans every message of `inner`.
fn encloses(outer: &FragmentSpec, inner: &FragmentSpec) -> bool {
    outer.from <= inner.from && inner.to <= outer.to
}

/// Whether `text` is an identifier: a letter, then letters, digits, `-` or `_`, at most 64.
pub(super) fn is_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic())
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        && text.len() <= 64
}

impl ChartSpec {
    /// A sequence diagram: participants with unique identifiers, messages between them, and
    /// fragments over runs of messages that nest without crossing.
    pub(super) fn validate_sequence(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a sequence diagram", "sequence.messages")?;
        let Some(sequence) = &self.sequence else {
            return Err(ChartError::new(
                "missing_sequence",
                "/sequence",
                "a sequence diagram requires a sequence block",
            ));
        };
        if sequence.participants.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/sequence/participants",
                "provide at least one participant",
            ));
        }
        if sequence.participants.len() > MAX_PARTICIPANTS {
            return Err(ChartError::new(
                "too_many_participants",
                "/sequence/participants",
                format!("at most {MAX_PARTICIPANTS} participants are supported"),
            ));
        }
        for (index, participant) in sequence.participants.iter().enumerate() {
            let path = format!("/sequence/participants/{index}");
            if !is_identifier(&participant.id) {
                return Err(ChartError::new(
                    "invalid_id",
                    format!("{path}/id"),
                    "use 1–64 ASCII letters, digits, hyphens, or underscores, starting with a letter",
                ));
            }
            if sequence.participants[..index]
                .iter()
                .any(|earlier| earlier.id == participant.id)
            {
                return Err(ChartError::new(
                    "duplicate_id",
                    format!("{path}/id"),
                    format!(
                        "the id \"{}\" is already taken by an earlier participant",
                        participant.id
                    ),
                ));
            }
            validate_text(&participant.label, &format!("{path}/label"), 60)?;
            if let Some(sublabel) = &participant.sublabel {
                validate_text(sublabel, &format!("{path}/sublabel"), 60)?;
            }
        }
        if sequence.messages.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/sequence/messages",
                "provide at least one message",
            ));
        }
        if sequence.messages.len() > MAX_MESSAGES {
            return Err(ChartError::new(
                "too_many_messages",
                "/sequence/messages",
                format!("at most {MAX_MESSAGES} messages are supported"),
            ));
        }
        for (index, message) in sequence.messages.iter().enumerate() {
            let path = format!("/sequence/messages/{index}");
            for (field, id) in [("from", &message.from), ("to", &message.to)] {
                if sequence.participant(id).is_none() {
                    return Err(ChartError::new(
                        "unknown_participant",
                        format!("{path}/{field}"),
                        format!("no participant has the id \"{id}\""),
                    ));
                }
            }
            validate_text(&message.label, &format!("{path}/label"), 80)?;
        }
        sequence.validate_fragments()?;
        Ok(Vec::new())
    }
}

impl SequenceSpec {
    /// Fragments over existing messages that nest without crossing, branches inside their frame.
    fn validate_fragments(&self) -> Result<(), ChartError> {
        if self.fragments.len() > MAX_FRAGMENTS {
            return Err(ChartError::new(
                "too_many_fragments",
                "/sequence/fragments",
                format!("at most {MAX_FRAGMENTS} fragments are supported"),
            ));
        }
        let messages = self.messages.len();
        for (index, fragment) in self.fragments.iter().enumerate() {
            let path = format!("/sequence/fragments/{index}");
            if fragment.to >= messages {
                return Err(ChartError::new(
                    "invalid_fragment",
                    format!("{path}/to"),
                    format!("to must name a message, from 0 to {}", messages - 1),
                ));
            }
            if fragment.from > fragment.to {
                return Err(ChartError::new(
                    "invalid_fragment",
                    format!("{path}/from"),
                    "from must not be greater than to",
                ));
            }
            if let Some(label) = &fragment.label {
                validate_text(label, &format!("{path}/label"), 60)?;
            }
            if !fragment.branches.is_empty() && fragment.kind != FragmentKind::Alt {
                return Err(ChartError::new(
                    "option_not_supported",
                    format!("{path}/else"),
                    "only an alt fragment has further branches",
                ));
            }
            let mut previous = fragment.from;
            for (branch_index, branch) in fragment.branches.iter().enumerate() {
                let branch_path = format!("{path}/else/{branch_index}");
                if branch.from <= previous || branch.from > fragment.to {
                    return Err(ChartError::new(
                        "invalid_fragment",
                        format!("{branch_path}/from"),
                        "a branch starts at a later message than the one before it, inside the frame",
                    ));
                }
                previous = branch.from;
                if let Some(label) = &branch.label {
                    validate_text(label, &format!("{branch_path}/label"), 60)?;
                }
            }
            for (other, earlier) in self.fragments[..index].iter().enumerate() {
                let disjoint = earlier.to < fragment.from || fragment.to < earlier.from;
                if !disjoint && !encloses(earlier, fragment) && !encloses(fragment, earlier) {
                    return Err(ChartError::new(
                        "fragments_cross",
                        format!("{path}/from"),
                        format!(
                            "this fragment overlaps fragment {other} without one enclosing the other"
                        ),
                    ));
                }
            }
            if self.depth(index) >= MAX_FRAGMENT_DEPTH {
                return Err(ChartError::new(
                    "invalid_fragment",
                    path,
                    format!("fragments nest at most {MAX_FRAGMENT_DEPTH} deep"),
                ));
            }
        }
        Ok(())
    }
}
