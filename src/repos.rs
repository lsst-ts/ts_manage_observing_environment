#[derive(clap::ValueEnum, Clone, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Repos {
    TsObservatoryControl,
    Atmospec,
    Spectractor,
    SummitExtras,
    SummitUtils,
    TsExternalscripts,
    TsObservingUtilities,
    TsStandardscripts,
    TsAuxtelStandardscripts,
    TsMaintelStandardscripts,
    TsWep,
    TsConfigOCS,
    TsConfigATTCS,
    TsConfigMTTCS,
    TsConfigScheduler,
}

impl Repos {
    pub fn get_name(&self) -> &str {
        match self {
            Repos::TsObservatoryControl => "ts_observatory_control",
            Repos::Atmospec => "atmospec",
            Repos::Spectractor => "Spectractor",
            Repos::SummitExtras => "summit_extras",
            Repos::SummitUtils => "summit_utils",
            Repos::TsExternalscripts => "ts_externalscripts",
            Repos::TsObservingUtilities => "ts_observing_utilities",
            Repos::TsStandardscripts => "ts_standardscripts",
            Repos::TsAuxtelStandardscripts => "ts_auxtel_standardscripts",
            Repos::TsMaintelStandardscripts => "ts_maintel_standardscripts",
            Repos::TsWep => "ts_wep",
            Repos::TsConfigOCS => "ts_config_ocs",
            Repos::TsConfigATTCS => "ts_config_attcs",
            Repos::TsConfigMTTCS => "ts_config_mttcs",
            Repos::TsConfigScheduler => "ts_config_scheduler",
        }
    }

    pub fn new_from_str(repository: &str) -> Option<Self> {
        match repository {
            "ts_observatory_control" => Some(Repos::TsObservatoryControl),
            "atmospec" => Some(Repos::Atmospec),
            "Spectractor" => Some(Repos::Spectractor),
            "summit_extras" => Some(Repos::SummitExtras),
            "summit_utils" => Some(Repos::SummitUtils),
            "ts_externalscripts" => Some(Repos::TsExternalscripts),
            "ts_observing_utilities" => Some(Repos::TsObservingUtilities),
            "ts_standardscripts" => Some(Repos::TsStandardscripts),
            "ts_auxtel_standardscripts" => Some(Repos::TsAuxtelStandardscripts),
            "ts_maintel_standardscripts" => Some(Repos::TsMaintelStandardscripts),
            "ts_wep" => Some(Repos::TsWep),
            "ts_config_ocs" => Some(Repos::TsConfigOCS),
            "ts_config_attcs" => Some(Repos::TsConfigATTCS),
            "ts_config_mttcs" => Some(Repos::TsConfigMTTCS),
            "ts_config_scheduler" => Some(Repos::TsConfigScheduler),
            _ => None,
        }
    }
}
