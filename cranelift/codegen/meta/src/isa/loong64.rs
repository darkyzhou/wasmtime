use crate::cdsl::{isa::TargetIsa, settings::SettingGroupBuilder};

pub(crate) fn define() -> TargetIsa {
    let mut settings = SettingGroupBuilder::new("loong64");

    settings.add_bool(
        "has_fp_sp",
        "Has single-precision floating-point support.",
        "",
        true,
    );
    settings.add_bool(
        "has_fp_dp",
        "Has double-precision floating-point support.",
        "",
        true,
    );
    settings.add_bool(
        "has_lsx",
        "Has Loongson SIMD Extension (LSX) support.",
        "",
        true,
    );
    settings.add_bool(
        "has_lasx",
        "Has Loongson Advanced SIMD Extension (LASX) support.",
        "",
        true,
    );

    TargetIsa::new("loong64", settings.build())
}
