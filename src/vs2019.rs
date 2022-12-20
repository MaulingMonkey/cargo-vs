#![forbid(unsafe_code)]

mod metadata;
mod run;

fn main() {
    run::run(run::Vs{
        nnnn:               "vs2019",
        sln_comment:        "Visual Studio Version 16",
        sln_vsv:            "16.0.33027.164",
        vcxproj_version:    "16.0",
        platform_toolset:   "v142",
    })
}
