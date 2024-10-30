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
    TsWep,
    TsConfigOCS,
    TsConfigATTCS,
    TsConfigMTTCS,
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
            Repos::TsWep => "ts_wep",
            Repos::TsConfigOCS => "ts_config_ocs",
            Repos::TsConfigATTCS => "ts_config_attcs",
            Repos::TsConfigMTTCS => "ts_config_mttcs",
        }
    }
}
