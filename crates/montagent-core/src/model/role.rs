//! Semantic audio roles for stem export (ADR-0204).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Semantic audio role for stem partitioning: dialogue, voiceover, music, sfx, foley (ADR-0204).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Dialogue,
    Voiceover,
    Music,
    Sfx,
    Foley,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Dialogue => "dialogue",
            Role::Voiceover => "voiceover",
            Role::Music => "music",
            Role::Sfx => "sfx",
            Role::Foley => "foley",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
