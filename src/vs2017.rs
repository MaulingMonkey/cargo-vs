#![forbid(unsafe_code)]

mod metadata;
mod run;

fn main() {
    run::run(run::Vs{
        nnnn:               "vs2017",
        sln_comment:        "Visual Studio 15",
        sln_vsv:            "15.0.28307.645",
        vcxproj_version:    "15.0",
        platform_toolset:   "v141",
    })
}
