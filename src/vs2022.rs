#![forbid(unsafe_code)]

mod metadata;
mod run;

fn main() {
    run::run(run::Vs{
        nnnn:               "vs2022",
        sln_comment:        "Visual Studio Version 17",
        sln_vsv:            "17.3.32825.248",
        vcxproj_version:    "16.0", // [sic]: VS2022 didn't bump this version in generated projects
        platform_toolset:   "v143",
    })
}
