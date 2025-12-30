//! STIX Vocabularies
//!
//! This module contains the open vocabulary types defined in the STIX specification.
//! These vocabularies provide standardized values for various properties while still
//! allowing custom values.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Macro to define an open vocabulary enum.
///
/// Open vocabularies have predefined values but also allow custom strings.
macro_rules! define_open_vocab {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $value:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
            /// Custom vocabulary value
            Custom(String),
        }

        impl $name {
            /// Get the string value of this vocabulary entry.
            pub fn as_str(&self) -> &str {
                match self {
                    $(
                        $name::$variant => $value,
                    )*
                    $name::Custom(s) => s,
                }
            }

            /// Get all standard vocabulary values.
            pub fn values() -> &'static [&'static str] {
                &[$($value),*]
            }

            /// Check if this is a standard value.
            pub fn is_standard(&self) -> bool {
                !matches!(self, $name::Custom(_))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                match s {
                    $(
                        $value => $name::$variant,
                    )*
                    other => $name::Custom(other.to_string()),
                }
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self::from(s.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                Ok(Self::from(s))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                // Return the first variant as default
                $name::Custom(String::new())
            }
        }
    };
}

// Attack Motivation
define_open_vocab! {
    /// Attack motivation vocabulary.
    AttackMotivation {
        /// Accidental - unintentional damage
        Accidental => "accidental",
        /// Coercion - forced to conduct attack
        Coercion => "coercion",
        /// Dominance - seeking power/control
        Dominance => "dominance",
        /// Ideology - political/social beliefs
        Ideology => "ideology",
        /// Notoriety - seeking fame/recognition
        Notoriety => "notoriety",
        /// Organizational gain - commercial/competitive advantage
        OrganizationalGain => "organizational-gain",
        /// Personal gain - individual profit
        PersonalGain => "personal-gain",
        /// Personal satisfaction - fun/thrill
        PersonalSatisfaction => "personal-satisfaction",
        /// Revenge - retaliation
        Revenge => "revenge",
        /// Unpredictable - unknown motivation
        Unpredictable => "unpredictable",
    }
}

// Attack Resource Level
define_open_vocab! {
    /// Attack resource level vocabulary.
    AttackResourceLevel {
        /// Individual - single attacker
        Individual => "individual",
        /// Club - small informal group
        Club => "club",
        /// Contest - hacking competition participants
        Contest => "contest",
        /// Team - organized group
        Team => "team",
        /// Organization - large organization
        Organization => "organization",
        /// Government - nation-state level
        Government => "government",
    }
}

// Identity Class
define_open_vocab! {
    /// Identity class vocabulary.
    IdentityClass {
        /// Individual person
        Individual => "individual",
        /// Group of people
        Group => "group",
        /// System or component
        System => "system",
        /// Organization
        Organization => "organization",
        /// Class of entities
        Class => "class",
        /// Unknown identity class
        Unknown => "unknown",
    }
}

// Implementation State
define_open_vocab! {
    /// Implementation state vocabulary.
    ImplementationState {
        /// External - implemented externally
        External => "external",
        /// Internal - implemented internally
        Internal => "internal",
        /// Pending - implementation pending
        Pending => "pending",
        /// Partial - partially implemented
        Partial => "partial",
        /// Full - fully implemented
        Full => "full",
    }
}

// Indicator Type
define_open_vocab! {
    /// Indicator type vocabulary.
    IndicatorType {
        /// Anomalous activity
        AnomalousActivity => "anomalous-activity",
        /// Anonymization
        Anonymization => "anonymization",
        /// Benign - known good
        Benign => "benign",
        /// Compromised - known compromised
        Compromised => "compromised",
        /// Malicious activity
        MaliciousActivity => "malicious-activity",
        /// Attribution - linked to threat actor
        Attribution => "attribution",
        /// Unknown indicator type
        Unknown => "unknown",
    }
}

// Industry Sector
define_open_vocab! {
    /// Industry sector vocabulary.
    IndustrySector {
        /// Agriculture
        Agriculture => "agriculture",
        /// Aerospace
        Aerospace => "aerospace",
        /// Automotive
        Automotive => "automotive",
        /// Chemical
        Chemical => "chemical",
        /// Commercial
        Commercial => "commercial",
        /// Communications
        Communications => "communications",
        /// Construction
        Construction => "construction",
        /// Dams
        Dams => "dams",
        /// Defense
        Defense => "defense",
        /// Education
        Education => "education",
        /// Emergency services
        EmergencyServices => "emergency-services",
        /// Energy
        Energy => "energy",
        /// Entertainment
        Entertainment => "entertainment",
        /// Financial Services
        FinancialServices => "financial-services",
        /// Government
        Government => "government",
        /// Government - local
        GovernmentLocal => "government-local",
        /// Government - national
        GovernmentNational => "government-national",
        /// Government - public services
        GovernmentPublicServices => "government-public-services",
        /// Government - regional
        GovernmentRegional => "government-regional",
        /// Healthcare
        Healthcare => "healthcare",
        /// Hospitality and leisure
        HospitalityLeisure => "hospitality-leisure",
        /// Infrastructure
        Infrastructure => "infrastructure",
        /// Insurance
        Insurance => "insurance",
        /// Manufacturing
        Manufacturing => "manufacturing",
        /// Mining
        Mining => "mining",
        /// Non-profit
        NonProfit => "non-profit",
        /// Nuclear
        Nuclear => "nuclear",
        /// Pharmaceuticals
        Pharmaceuticals => "pharmaceuticals",
        /// Retail
        Retail => "retail",
        /// Technology
        Technology => "technology",
        /// Telecommunications
        Telecommunications => "telecommunications",
        /// Transportation
        Transportation => "transportation",
        /// Utilities
        Utilities => "utilities",
        /// Water
        Water => "water",
    }
}

// Malware Type
define_open_vocab! {
    /// Malware type vocabulary.
    MalwareType {
        /// Adware
        Adware => "adware",
        /// Backdoor
        Backdoor => "backdoor",
        /// Bot
        Bot => "bot",
        /// Bootkit
        Bootkit => "bootkit",
        /// DDoS bot
        DdosBot => "ddos",
        /// Downloader
        Downloader => "downloader",
        /// Dropper
        Dropper => "dropper",
        /// Exploit kit
        ExploitKit => "exploit-kit",
        /// Keylogger
        Keylogger => "keylogger",
        /// Ransomware
        Ransomware => "ransomware",
        /// Remote access trojan
        RemoteAccessTrojan => "remote-access-trojan",
        /// Resource exploitation
        ResourceExploitation => "resource-exploitation",
        /// Rogue security software
        RogueSecuritySoftware => "rogue-security-software",
        /// Rootkit
        Rootkit => "rootkit",
        /// Screen capture
        ScreenCapture => "screen-capture",
        /// Spyware
        Spyware => "spyware",
        /// Trojan
        Trojan => "trojan",
        /// Unknown malware type
        Unknown => "unknown",
        /// Virus
        Virus => "virus",
        /// Webshell
        Webshell => "webshell",
        /// Wiper
        Wiper => "wiper",
        /// Worm
        Worm => "worm",
    }
}

// Malware Capabilities
define_open_vocab! {
    /// Malware capabilities vocabulary.
    MalwareCapability {
        /// Accesses remote machines
        AccessesRemoteMachines => "accesses-remote-machines",
        /// Anti-debugging
        AntiDebugging => "anti-debugging",
        /// Anti-disassembly
        AntiDisassembly => "anti-disassembly",
        /// Anti-emulation
        AntiEmulation => "anti-emulation",
        /// Anti-memory forensics
        AntiMemoryForensics => "anti-memory-forensics",
        /// Anti-sandbox
        AntiSandbox => "anti-sandbox",
        /// Anti-VM
        AntiVm => "anti-vm",
        /// Captures input peripherals
        CapturesInputPeripherals => "captures-input-peripherals",
        /// Captures output peripherals
        CapturesOutputPeripherals => "captures-output-peripherals",
        /// Captures system state data
        CapturesSystemStateData => "captures-system-state-data",
        /// Cleans traces of infection
        CleansTracesOfInfection => "cleans-traces-of-infection",
        /// Commits fraud
        CommitsFraud => "commits-fraud",
        /// Communicates with C2
        CommunicatesWithC2 => "communicates-with-c2",
        /// Compromises data availability
        CompromisesDataAvailability => "compromises-data-availability",
        /// Compromises data integrity
        CompromisesDataIntegrity => "compromises-data-integrity",
        /// Compromises system availability
        CompromisesSystemAvailability => "compromises-system-availability",
        /// Controls local machine
        ControlsLocalMachine => "controls-local-machine",
        /// Degrades security software
        DegradeSecuritySoftware => "degrades-security-software",
        /// Degrades system updates
        DegradeSystemUpdates => "degrades-system-updates",
        /// Determines C2 server
        DeterminesC2Server => "determines-c2-server",
        /// Emails spam
        EmailsSpam => "emails-spam",
        /// Escalates privileges
        EscalatesPrivileges => "escalates-privileges",
        /// Evades AV
        EvadesAv => "evades-av",
        /// Exfiltrates data
        ExfiltratesData => "exfiltrates-data",
        /// Fingerprints host
        FingerprintsHost => "fingerprints-host",
        /// Hides artifacts
        HidesArtifacts => "hides-artifacts",
        /// Hides executing code
        HidesExecutingCode => "hides-executing-code",
        /// Infects files
        InfectsFiles => "infects-files",
        /// Infects remote machines
        InfectsRemoteMachines => "infects-remote-machines",
        /// Installs other components
        InstallsOtherComponents => "installs-other-components",
        /// Persists after reboot
        PersistsAfterSystemReboot => "persists-after-system-reboot",
        /// Prevents artifact access
        PreventsArtifactAccess => "prevents-artifact-access",
        /// Prevents artifact deletion
        PreventsArtifactDeletion => "prevents-artifact-deletion",
        /// Probes network
        ProbesNetworkEnvironment => "probes-network-environment",
        /// Self-modifies
        SelfModifies => "self-modifies",
        /// Steals authentication credentials
        StealsAuthenticationCredentials => "steals-authentication-credentials",
        /// Violates system operational integrity
        ViolatesSystemOperationalIntegrity => "violates-system-operational-integrity",
    }
}

// Pattern Type
define_open_vocab! {
    /// Pattern type vocabulary.
    PatternType {
        /// STIX pattern
        Stix => "stix",
        /// PCRE pattern
        Pcre => "pcre",
        /// Sigma pattern
        Sigma => "sigma",
        /// Snort pattern
        Snort => "snort",
        /// Suricata pattern
        Suricata => "suricata",
        /// YARA pattern
        Yara => "yara",
    }
}

// Report Type
define_open_vocab! {
    /// Report type vocabulary.
    ReportType {
        /// Attack pattern report
        AttackPattern => "attack-pattern",
        /// Campaign report
        Campaign => "campaign",
        /// Identity report
        Identity => "identity",
        /// Indicator report
        Indicator => "indicator",
        /// Intrusion set report
        IntrusionSet => "intrusion-set",
        /// Malware report
        Malware => "malware",
        /// Observed data report
        ObservedData => "observed-data",
        /// Threat actor report
        ThreatActor => "threat-actor",
        /// Threat report
        ThreatReport => "threat-report",
        /// Tool report
        Tool => "tool",
        /// Vulnerability report
        Vulnerability => "vulnerability",
    }
}

// Threat Actor Type
define_open_vocab! {
    /// Threat actor type vocabulary.
    ThreatActorType {
        /// Activist
        Activist => "activist",
        /// Competitor
        Competitor => "competitor",
        /// Crime syndicate
        CrimeSyndicate => "crime-syndicate",
        /// Criminal
        Criminal => "criminal",
        /// Hacker
        Hacker => "hacker",
        /// Insider accidental
        InsiderAccidental => "insider-accidental",
        /// Insider disgruntled
        InsiderDisgruntled => "insider-disgruntled",
        /// Nation state
        NationState => "nation-state",
        /// Sensationalist
        Sensationalist => "sensationalist",
        /// Spy
        Spy => "spy",
        /// Terrorist
        Terrorist => "terrorist",
        /// Unknown threat actor type
        Unknown => "unknown",
    }
}

// Threat Actor Role
define_open_vocab! {
    /// Threat actor role vocabulary.
    ThreatActorRole {
        /// Agent - performs attacks
        Agent => "agent",
        /// Director - directs attacks
        Director => "director",
        /// Independent - operates alone
        Independent => "independent",
        /// Infrastructure architect/administrator
        InfrastructureArchitect => "infrastructure-architect",
        /// Infrastructure operator
        InfrastructureOperator => "infrastructure-operator",
        /// Malware author
        MalwareAuthor => "malware-author",
        /// Sponsor - funds attacks
        Sponsor => "sponsor",
    }
}

// Threat Actor Sophistication
define_open_vocab! {
    /// Threat actor sophistication vocabulary.
    ThreatActorSophistication {
        /// None - no technical skills
        None => "none",
        /// Minimal - limited skills
        Minimal => "minimal",
        /// Intermediate - moderate skills
        Intermediate => "intermediate",
        /// Advanced - high skills
        Advanced => "advanced",
        /// Expert - expert skills
        Expert => "expert",
        /// Innovator - creates new techniques
        Innovator => "innovator",
        /// Strategic - strategic planning
        Strategic => "strategic",
    }
}

// Tool Type
define_open_vocab! {
    /// Tool type vocabulary.
    ToolType {
        /// Denial of service tool
        DenialOfService => "denial-of-service",
        /// Exploitation tool
        Exploitation => "exploitation",
        /// Information gathering tool
        InformationGathering => "information-gathering",
        /// Network capture tool
        NetworkCapture => "network-capture",
        /// Credential exploitation tool
        CredentialExploitation => "credential-exploitation",
        /// Remote access tool
        RemoteAccess => "remote-access",
        /// Vulnerability scanning tool
        VulnerabilityScanning => "vulnerability-scanning",
        /// Unknown tool type
        Unknown => "unknown",
    }
}

// Hash Algorithm
define_open_vocab! {
    /// Hash algorithm vocabulary.
    HashAlgorithm {
        /// MD5
        Md5 => "MD5",
        /// SHA-1
        Sha1 => "SHA-1",
        /// SHA-256
        Sha256 => "SHA-256",
        /// SHA-512
        Sha512 => "SHA-512",
        /// SHA3-256
        Sha3_256 => "SHA3-256",
        /// SHA3-512
        Sha3_512 => "SHA3-512",
        /// SSDEEP
        Ssdeep => "SSDEEP",
        /// TLSH
        Tlsh => "TLSH",
    }
}

// Encryption Algorithm
define_open_vocab! {
    /// Encryption algorithm vocabulary.
    EncryptionAlgorithm {
        /// AES-256-GCM
        Aes256Gcm => "AES-256-GCM",
        /// ChaCha20-Poly1305
        ChaCha20Poly1305 => "ChaCha20-Poly1305",
        /// mime-type-indicated
        MimeTypeIndicated => "mime-type-indicated",
    }
}

// Windows Registry Datatype
define_open_vocab! {
    /// Windows registry datatype vocabulary.
    WindowsRegistryDatatype {
        /// REG_NONE
        RegNone => "REG_NONE",
        /// REG_SZ
        RegSz => "REG_SZ",
        /// REG_EXPAND_SZ
        RegExpandSz => "REG_EXPAND_SZ",
        /// REG_BINARY
        RegBinary => "REG_BINARY",
        /// REG_DWORD
        RegDword => "REG_DWORD",
        /// REG_DWORD_BIG_ENDIAN
        RegDwordBigEndian => "REG_DWORD_BIG_ENDIAN",
        /// REG_DWORD_LITTLE_ENDIAN
        RegDwordLittleEndian => "REG_DWORD_LITTLE_ENDIAN",
        /// REG_LINK
        RegLink => "REG_LINK",
        /// REG_MULTI_SZ
        RegMultiSz => "REG_MULTI_SZ",
        /// REG_RESOURCE_LIST
        RegResourceList => "REG_RESOURCE_LIST",
        /// REG_FULL_RESOURCE_DESCRIPTOR
        RegFullResourceDescriptor => "REG_FULL_RESOURCE_DESCRIPTOR",
        /// REG_RESOURCE_REQUIREMENTS_LIST
        RegResourceRequirementsList => "REG_RESOURCE_REQUIREMENTS_LIST",
        /// REG_QWORD
        RegQword => "REG_QWORD",
        /// REG_INVALID_TYPE
        RegInvalidType => "REG_INVALID_TYPE",
    }
}

// Account Type
define_open_vocab! {
    /// Account type vocabulary.
    AccountType {
        /// Facebook
        Facebook => "facebook",
        /// LDAP
        Ldap => "ldap",
        /// NIS
        Nis => "nis",
        /// OpenID
        Openid => "openid",
        /// RADIUS
        Radius => "radius",
        /// Skype
        Skype => "skype",
        /// TACACS
        Tacacs => "tacacs",
        /// Twitter
        Twitter => "twitter",
        /// Unix
        Unix => "unix",
        /// Windows domain
        WindowsDomain => "windows-domain",
        /// Windows local
        WindowsLocal => "windows-local",
    }
}

// Opinion enum (not an open vocab)
define_open_vocab! {
    /// Opinion value vocabulary.
    OpinionValue {
        /// Strongly disagree
        StronglyDisagree => "strongly-disagree",
        /// Disagree
        Disagree => "disagree",
        /// Neutral
        Neutral => "neutral",
        /// Agree
        Agree => "agree",
        /// Strongly agree
        StronglyAgree => "strongly-agree",
    }
}

// Grouping Context
define_open_vocab! {
    /// Grouping context vocabulary.
    GroupingContext {
        /// Suspicious activity
        SuspiciousActivity => "suspicious-activity",
        /// Malware analysis
        MalwareAnalysis => "malware-analysis",
        /// Unspecified
        Unspecified => "unspecified",
    }
}

// Infrastructure Type
define_open_vocab! {
    /// Infrastructure type vocabulary.
    InfrastructureType {
        /// Amplification
        Amplification => "amplification",
        /// Anonymization
        Anonymization => "anonymization",
        /// Botnet
        Botnet => "botnet",
        /// Command and control
        CommandAndControl => "command-and-control",
        /// Control system
        ControlSystem => "control-system",
        /// Exfiltration
        Exfiltration => "exfiltration",
        /// Firewall
        Firewall => "firewall",
        /// Hosting malware
        HostingMalware => "hosting-malware",
        /// Hosting target lists
        HostingTargetLists => "hosting-target-lists",
        /// Phishing
        Phishing => "phishing",
        /// Reconnaissance
        Reconnaissance => "reconnaissance",
        /// Routers switches
        RoutersSwitches => "routers-switches",
        /// Staging
        Staging => "staging",
        /// Workstation
        Workstation => "workstation",
        /// Unknown
        Unknown => "unknown",
    }
}

// Windows PE Binary Type
define_open_vocab! {
    /// Windows PE binary type vocabulary.
    WindowsPeBinaryType {
        /// Dynamically linked library (DLL)
        Dll => "dll",
        /// Executable
        Exe => "exe",
        /// System driver
        Sys => "sys",
    }
}

// Network Socket Address Family
define_open_vocab! {
    /// Network socket address family vocabulary.
    NetworkSocketAddressFamily {
        /// Unspecified
        AfUnspec => "AF_UNSPEC",
        /// IPv4 address family
        AfInet => "AF_INET",
        /// IPX protocol
        AfIpx => "AF_IPX",
        /// AppleTalk protocols
        AfAppletalk => "AF_APPLETALK",
        /// NetBIOS protocols
        AfNetbios => "AF_NETBIOS",
        /// IPv6 address family
        AfInet6 => "AF_INET6",
        /// IrDA (Infrared)
        AfIrda => "AF_IRDA",
        /// Bluetooth
        AfBth => "AF_BTH",
    }
}

// Network Socket Type
define_open_vocab! {
    /// Network socket type vocabulary.
    NetworkSocketType {
        /// Datagram socket
        SockDgram => "SOCK_DGRAM",
        /// Raw socket
        SockRaw => "SOCK_RAW",
        /// Reliable datagram socket
        SockRdm => "SOCK_RDM",
        /// Sequenced packet socket
        SockSeqpacket => "SOCK_SEQPACKET",
        /// Stream socket
        SockStream => "SOCK_STREAM",
    }
}

// Windows Integrity Level
define_open_vocab! {
    /// Windows integrity level vocabulary.
    WindowsIntegrityLevel {
        /// Low integrity level
        Low => "low",
        /// Medium integrity level
        Medium => "medium",
        /// High integrity level
        High => "high",
        /// System integrity level
        System => "system",
    }
}

// Windows Service Start Type
define_open_vocab! {
    /// Windows service start type vocabulary.
    WindowsServiceStartType {
        /// Auto start
        ServiceAutoStart => "SERVICE_AUTO_START",
        /// Boot start
        ServiceBootStart => "SERVICE_BOOT_START",
        /// Demand start
        ServiceDemandStart => "SERVICE_DEMAND_START",
        /// Disabled
        ServiceDisabled => "SERVICE_DISABLED",
        /// System alert
        ServiceSystemAlert => "SERVICE_SYSTEM_ALERT",
    }
}

// Windows Service Type
define_open_vocab! {
    /// Windows service type vocabulary.
    WindowsServiceType {
        /// File system driver
        ServiceFileSystemDriver => "SERVICE_FILE_SYSTEM_DRIVER",
        /// Kernel driver
        ServiceKernelDriver => "SERVICE_KERNEL_DRIVER",
        /// Win32 own process
        ServiceWin32OwnProcess => "SERVICE_WIN32_OWN_PROCESS",
        /// Win32 share process
        ServiceWin32ShareProcess => "SERVICE_WIN32_SHARE_PROCESS",
    }
}

// Windows Service Status
define_open_vocab! {
    /// Windows service status vocabulary.
    WindowsServiceStatus {
        /// Continue pending
        ServiceContinuePending => "SERVICE_CONTINUE_PENDING",
        /// Pause pending
        ServicePausePending => "SERVICE_PAUSE_PENDING",
        /// Paused
        ServicePaused => "SERVICE_PAUSED",
        /// Running
        ServiceRunning => "SERVICE_RUNNING",
        /// Start pending
        ServiceStartPending => "SERVICE_START_PENDING",
        /// Stop pending
        ServiceStopPending => "SERVICE_STOP_PENDING",
        /// Stopped
        ServiceStopped => "SERVICE_STOPPED",
    }
}

// Malware Analysis Result
define_open_vocab! {
    /// Malware analysis result vocabulary.
    MalwareAnalysisResult {
        /// Malicious
        Malicious => "malicious",
        /// Suspicious
        Suspicious => "suspicious",
        /// Benign
        Benign => "benign",
        /// Unknown
        Unknown => "unknown",
    }
}

// Region
define_open_vocab! {
    /// Geographic region vocabulary.
    Region {
        /// Africa
        Africa => "africa",
        /// Eastern Africa
        EasternAfrica => "eastern-africa",
        /// Middle Africa
        MiddleAfrica => "middle-africa",
        /// Northern Africa
        NorthernAfrica => "northern-africa",
        /// Southern Africa
        SouthernAfrica => "southern-africa",
        /// Western Africa
        WesternAfrica => "western-africa",
        /// Americas
        Americas => "americas",
        /// Caribbean
        Caribbean => "caribbean",
        /// Central America
        CentralAmerica => "central-america",
        /// Latin America Caribbean
        LatinAmericaCaribbean => "latin-america-caribbean",
        /// Northern America
        NorthernAmerica => "northern-america",
        /// South America
        SouthAmerica => "south-america",
        /// Asia
        Asia => "asia",
        /// Central Asia
        CentralAsia => "central-asia",
        /// Eastern Asia
        EasternAsia => "eastern-asia",
        /// Southern Asia
        SouthernAsia => "southern-asia",
        /// South-Eastern Asia
        SouthEasternAsia => "south-eastern-asia",
        /// Western Asia
        WesternAsia => "western-asia",
        /// Europe
        Europe => "europe",
        /// Eastern Europe
        EasternEurope => "eastern-europe",
        /// Northern Europe
        NorthernEurope => "northern-europe",
        /// Southern Europe
        SouthernEurope => "southern-europe",
        /// Western Europe
        WesternEurope => "western-europe",
        /// Oceania
        Oceania => "oceania",
        /// Antarctica
        Antarctica => "antarctica",
        /// Australia New Zealand
        AustraliaNewZealand => "australia-new-zealand",
        /// Melanesia
        Melanesia => "melanesia",
        /// Micronesia
        Micronesia => "micronesia",
        /// Polynesia
        Polynesia => "polynesia",
    }
}

// Extension Type
define_open_vocab! {
    /// Extension type vocabulary.
    ExtensionType {
        /// New SDO extension
        NewSdo => "new-sdo",
        /// New SCO extension
        NewSco => "new-sco",
        /// New SRO extension
        NewSro => "new-sro",
        /// Property extension
        PropertyExtension => "property-extension",
        /// Top-level property extension
        ToplevelPropertyExtension => "toplevel-property-extension",
    }
}

// Implementation Language
define_open_vocab! {
    /// Implementation language vocabulary.
    ImplementationLanguage {
        /// AppleScript
        Applescript => "applescript",
        /// Bash
        Bash => "bash",
        /// C
        C => "c",
        /// C++
        Cplusplus => "c++",
        /// C#
        Csharp => "c#",
        /// Go
        Go => "go",
        /// Java
        Java => "java",
        /// JavaScript
        Javascript => "javascript",
        /// Lua
        Lua => "lua",
        /// Objective-C
        ObjectiveC => "objective-c",
        /// Perl
        Perl => "perl",
        /// PHP
        Php => "php",
        /// PowerShell
        Powershell => "powershell",
        /// Python
        Python => "python",
        /// Ruby
        Ruby => "ruby",
        /// Scala
        Scala => "scala",
        /// Swift
        Swift => "swift",
        /// TypeScript
        Typescript => "typescript",
        /// Visual Basic
        VisualBasic => "visual-basic",
        /// x86 32-bit assembly
        X86_32 => "x86-32",
        /// x86 64-bit assembly
        X86_64 => "x86-64",
    }
}

// Processor Architecture
define_open_vocab! {
    /// Processor architecture vocabulary.
    ProcessorArchitecture {
        /// Alpha
        Alpha => "alpha",
        /// ARM
        Arm => "arm",
        /// IA-64 (Itanium)
        Ia64 => "ia-64",
        /// MIPS
        Mips => "mips",
        /// PowerPC
        Powerpc => "powerpc",
        /// SPARC
        Sparc => "sparc",
        /// x86
        X86 => "x86",
        /// x86-64
        X86_64 => "x86-64",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_motivation() {
        let motivation = AttackMotivation::Ideology;
        assert_eq!(motivation.as_str(), "ideology");
        assert!(motivation.is_standard());
    }

    #[test]
    fn test_custom_value() {
        let custom = AttackMotivation::Custom("custom-value".to_string());
        assert_eq!(custom.as_str(), "custom-value");
        assert!(!custom.is_standard());
    }

    #[test]
    fn test_from_str() {
        let motivation: AttackMotivation = "ideology".into();
        assert!(matches!(motivation, AttackMotivation::Ideology));

        let custom: AttackMotivation = "unknown-value".into();
        assert!(matches!(custom, AttackMotivation::Custom(_)));
    }

    #[test]
    fn test_serialization() {
        let pattern_type = PatternType::Stix;
        let json = serde_json::to_string(&pattern_type).unwrap();
        assert_eq!(json, "\"stix\"");

        let parsed: PatternType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, PatternType::Stix);
    }
}
