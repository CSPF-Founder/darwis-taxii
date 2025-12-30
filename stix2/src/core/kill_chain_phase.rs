//! Kill chain phase representation.
//!
//! Kill chain phases describe stages of an attack as defined by various
//! kill chain frameworks like Lockheed Martin Cyber Kill Chain or MITRE ATT&CK.

use serde::{Deserialize, Serialize};

/// A phase in a kill chain.
///
/// Kill chain phases represent a stage in an attack chain. They are used
/// to describe where in the attack lifecycle a particular technique,
/// tool, or malware might be employed.
///
/// # Example
///
/// ```rust
/// use stix2::core::KillChainPhase;
///
/// // Lockheed Martin Cyber Kill Chain phase
/// let phase = KillChainPhase::new("lockheed-martin-cyber-kill-chain", "exploitation");
///
/// // MITRE ATT&CK phase
/// let mitre_phase = KillChainPhase::mitre_attack("initial-access");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KillChainPhase {
    /// The name of the kill chain (e.g., "lockheed-martin-cyber-kill-chain").
    pub kill_chain_name: String,

    /// The name of the phase (e.g., "reconnaissance", "weaponization").
    pub phase_name: String,
}

impl KillChainPhase {
    /// Create a new kill chain phase.
    ///
    /// # Arguments
    ///
    /// * `kill_chain_name` - The name of the kill chain
    /// * `phase_name` - The name of the phase within the kill chain
    pub fn new(kill_chain_name: impl Into<String>, phase_name: impl Into<String>) -> Self {
        Self {
            kill_chain_name: kill_chain_name.into(),
            phase_name: phase_name.into(),
        }
    }

    /// Create a Lockheed Martin Cyber Kill Chain phase.
    ///
    /// # Arguments
    ///
    /// * `phase_name` - The phase name (e.g., "reconnaissance", "weaponization")
    pub fn lockheed_martin(phase_name: impl Into<String>) -> Self {
        Self::new("lockheed-martin-cyber-kill-chain", phase_name)
    }

    /// Create a MITRE ATT&CK phase.
    ///
    /// # Arguments
    ///
    /// * `tactic` - The ATT&CK tactic name (e.g., "initial-access", "execution")
    pub fn mitre_attack(tactic: impl Into<String>) -> Self {
        Self::new("mitre-attack", tactic)
    }

    /// Check if this phase belongs to a specific kill chain.
    pub fn is_kill_chain(&self, name: &str) -> bool {
        self.kill_chain_name.eq_ignore_ascii_case(name)
    }

    /// Check if this is a Lockheed Martin Cyber Kill Chain phase.
    pub fn is_lockheed_martin(&self) -> bool {
        self.is_kill_chain("lockheed-martin-cyber-kill-chain")
    }

    /// Check if this is a MITRE ATT&CK phase.
    pub fn is_mitre_attack(&self) -> bool {
        self.is_kill_chain("mitre-attack")
    }
}

/// Lockheed Martin Cyber Kill Chain phases.
pub mod lockheed_martin {
    use super::KillChainPhase;

    /// Reconnaissance phase.
    pub fn reconnaissance() -> KillChainPhase {
        KillChainPhase::lockheed_martin("reconnaissance")
    }

    /// Weaponization phase.
    pub fn weaponization() -> KillChainPhase {
        KillChainPhase::lockheed_martin("weaponization")
    }

    /// Delivery phase.
    pub fn delivery() -> KillChainPhase {
        KillChainPhase::lockheed_martin("delivery")
    }

    /// Exploitation phase.
    pub fn exploitation() -> KillChainPhase {
        KillChainPhase::lockheed_martin("exploitation")
    }

    /// Installation phase.
    pub fn installation() -> KillChainPhase {
        KillChainPhase::lockheed_martin("installation")
    }

    /// Command and Control phase.
    pub fn command_and_control() -> KillChainPhase {
        KillChainPhase::lockheed_martin("command-and-control")
    }

    /// Actions on Objectives phase.
    pub fn actions_on_objectives() -> KillChainPhase {
        KillChainPhase::lockheed_martin("actions-on-objectives")
    }
}

/// MITRE ATT&CK tactics (kill chain phases).
pub mod mitre_attack {
    use super::KillChainPhase;

    /// Reconnaissance tactic.
    pub fn reconnaissance() -> KillChainPhase {
        KillChainPhase::mitre_attack("reconnaissance")
    }

    /// Resource Development tactic.
    pub fn resource_development() -> KillChainPhase {
        KillChainPhase::mitre_attack("resource-development")
    }

    /// Initial Access tactic.
    pub fn initial_access() -> KillChainPhase {
        KillChainPhase::mitre_attack("initial-access")
    }

    /// Execution tactic.
    pub fn execution() -> KillChainPhase {
        KillChainPhase::mitre_attack("execution")
    }

    /// Persistence tactic.
    pub fn persistence() -> KillChainPhase {
        KillChainPhase::mitre_attack("persistence")
    }

    /// Privilege Escalation tactic.
    pub fn privilege_escalation() -> KillChainPhase {
        KillChainPhase::mitre_attack("privilege-escalation")
    }

    /// Defense Evasion tactic.
    pub fn defense_evasion() -> KillChainPhase {
        KillChainPhase::mitre_attack("defense-evasion")
    }

    /// Credential Access tactic.
    pub fn credential_access() -> KillChainPhase {
        KillChainPhase::mitre_attack("credential-access")
    }

    /// Discovery tactic.
    pub fn discovery() -> KillChainPhase {
        KillChainPhase::mitre_attack("discovery")
    }

    /// Lateral Movement tactic.
    pub fn lateral_movement() -> KillChainPhase {
        KillChainPhase::mitre_attack("lateral-movement")
    }

    /// Collection tactic.
    pub fn collection() -> KillChainPhase {
        KillChainPhase::mitre_attack("collection")
    }

    /// Command and Control tactic.
    pub fn command_and_control() -> KillChainPhase {
        KillChainPhase::mitre_attack("command-and-control")
    }

    /// Exfiltration tactic.
    pub fn exfiltration() -> KillChainPhase {
        KillChainPhase::mitre_attack("exfiltration")
    }

    /// Impact tactic.
    pub fn impact() -> KillChainPhase {
        KillChainPhase::mitre_attack("impact")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_kill_chain_phase() {
        let phase = KillChainPhase::new("custom-kill-chain", "phase-1");
        assert_eq!(phase.kill_chain_name, "custom-kill-chain");
        assert_eq!(phase.phase_name, "phase-1");
    }

    #[test]
    fn test_lockheed_martin_phase() {
        let phase = lockheed_martin::exploitation();
        assert!(phase.is_lockheed_martin());
        assert_eq!(phase.phase_name, "exploitation");
    }

    #[test]
    fn test_mitre_attack_phase() {
        let phase = mitre_attack::initial_access();
        assert!(phase.is_mitre_attack());
        assert_eq!(phase.phase_name, "initial-access");
    }

    #[test]
    fn test_serialization() {
        let phase = KillChainPhase::new("test-chain", "test-phase");
        let json = serde_json::to_string(&phase).unwrap();
        let parsed: KillChainPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(phase, parsed);
    }
}
