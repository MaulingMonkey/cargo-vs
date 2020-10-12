use crate::*;

use uuid::Uuid;

use std::ffi::*;
use std::fs::File;
use std::io::{self, Write};
use std::path::{PathBuf};
use std::process::exit;



#[derive(Clone, Copy)]
struct Arch {
    pub vs_sln_name:    &'static str,
    pub vs_proj_name:   &'static str,
    pub cargo_name:     &'static str,
}

const ARCHES : &'static [Arch] = &[
    Arch { vs_sln_name: "x64", vs_proj_name: "x64",   cargo_name: "x86_64" },
    Arch { vs_sln_name: "x86", vs_proj_name: "Win32", cargo_name: "i686"   },
];

#[derive(Clone, Copy)]
struct Config {
    pub vs_name:            &'static str,
    pub target_dir:         &'static str,
    pub cargo_build_flags:  &'static str,
}

const CONFIGS : &'static [Config] = &[
    Config { vs_name: "Debug",   target_dir: "debug",   cargo_build_flags: "" },
    Config { vs_name: "Release", target_dir: "release", cargo_build_flags: " --release" },
];

const GITIGNORE : &'static str = concat!(
    "/.vs/\r\n",
    "/int/\r\n",
    "/**/*.log\r\n",
    "/**/*.suo\r\n",
    "/**/*.user\r\n",
    "/**/*.VC.db\r\n",
    "*"
);



pub fn run() {
    let meta = metadata::Root::get().unwrap_or_else(|err| { eprintln!("error parsing `cargo metadata`: {}", err); exit(1) });
    let vs = create_vs_dir(&meta).unwrap_or_else(|err| { eprintln!("error creating vs directory: {}", err); exit(1) });
    let context = Context { meta, vs, _non_exhaustive: () };

    let mut errors = false;
    create_vs_sln(&context).unwrap_or_else(|err| { eprintln!("{}", err); errors = true; });
    if errors { exit(1) }
}

fn create_vs_dir(meta: &metadata::Root) -> io::Result<PathBuf> {
    let vs = meta.workspace.dir.join("vs");
    match std::fs::create_dir(&vs) {
        Ok(()) => {
            std::fs::write(vs.join(".gitignore"), GITIGNORE).map_err(|err| io::Error::new(err.kind(), format!("unable to create .gitignore: {}", err)))?; // XXX: remap err for more context?
            Ok(vs)
        },
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(vs),
        Err(err) => Err(err),
    }
}

struct Context {
    meta:   metadata::Root,
    vs:     PathBuf,

    _non_exhaustive: ()
}

fn create_vs_sln(context: &Context) -> io::Result<()> {
    let Context { meta, vs, .. } = context;

    let vs_nnnn = "vs2017";

    //let mut sln = meta.workspace.dir.file_name().ok_or(io::ErrorKind::InvalidInput)?.to_os_string();
    //sln.push(OsStr::new("-"));
    let mut sln = OsString::new();
    sln.push(OsStr::new(vs_nnnn));
    sln.push(OsStr::new(".sln"));
    let sln = vs.join(sln);

    let projects_dir = vs.join(vs_nnnn);
    std::fs::create_dir_all(&projects_dir)?;

    let mut o = File::create(&sln).map_err(|err| io::Error::new(err.kind(), format!("error creating {}: {}", sln.display(), err)))?;
    o.write_all(&[0xEF, 0xBB, 0xBF, b'\r', b'\n'])?; // UTF-8 Psuedo-BOM + CR + LF

    // VS2017 - I should add support for more editions + SKUs eventually...
    write!(o, "Microsoft Visual Studio Solution File, Format Version 12.00\r\n")?;
    write!(o, "# Visual Studio 15\r\n")?;
    write!(o, "VisualStudioVersion = 15.0.28307.645\r\n")?;
    write!(o, "MinimumVisualStudioVersion = 10.0.40219.1\r\n")?;
    let uuid_sln = Uuid::new_v5(&Uuid::NAMESPACE_OID, sln.to_string_lossy().as_bytes());

    let mut uuid_buf = [b'!'; 40];

    for package in meta.packages.iter() {
        if !meta.workspace_members.contains(&package.id) { continue }
        for target in package.targets.iter() {
            if !target.kind.iter().any(|kind| ["bin", "example"].contains(&&**kind)) { continue }
            let uuid_project_type = "8BC9CEB8-8B4A-11D0-8D11-00A0C91BC942"; // makefile
            let uuid_project = project_guid(context, &package.id, &target);

            write!(o, "Project(\"{{{uuid_project_type}}}\") = \"{target_name}\", \"{vs_nnnn}\\{target_name}.vcxproj\", \"{{{uuid_project}}}\"\r\n", uuid_project_type=uuid_project_type, target_name=target.name, vs_nnnn=vs_nnnn, uuid_project=uuid_project.to_hyphenated_ref().encode_upper(&mut uuid_buf))?;
            write!(o, "EndProject\r\n")?;
        }
    }

    write!(o, "Global\r\n")?;

    write!(o, "\tGlobalSection(SolutionConfigurationPlatforms) = preSolution\r\n")?;
    for config in ["Debug", "Release"].iter().copied() {
        for arch in ["x64", "x86"].iter().copied() {
            write!(o, "\t\t{config}|{arch} = {config}|{arch}\r\n", config=config, arch=arch)?;
        }
    }
    write!(o, "\tEndGlobalSection\r\n")?;

    write!(o, "\tGlobalSection(ProjectConfigurationPlatforms) = postSolution\r\n")?;
    for package in meta.packages.iter() {
        if !meta.workspace_members.contains(&package.id) { continue }
        for target in package.targets.iter() {
            if !target.kind.iter().any(|kind| ["bin", "example"].contains(&&**kind)) { continue }
            let uuid_project = project_guid(context, &package.id, &target);
            for config in CONFIGS.iter().copied() {
                for arch in ARCHES.iter().copied() {
                    let sln_arch = arch.vs_sln_name;
                    let proj_arch = arch.vs_proj_name;
                    write!(o, "\t\t{{{uuid_project}}}.{config}|{sln_arch}.ActiveCfg = {config}|{proj_arch}\r\n", uuid_project=uuid_project.to_hyphenated_ref().encode_upper(&mut uuid_buf), config=config.vs_name, sln_arch=sln_arch, proj_arch=proj_arch)?;
                    write!(o, "\t\t{{{uuid_project}}}.{config}|{sln_arch}.Build.0 = {config}|{proj_arch}\r\n", uuid_project=uuid_project.to_hyphenated_ref().encode_upper(&mut uuid_buf), config=config.vs_name, sln_arch=sln_arch, proj_arch=proj_arch)?;
                }
            }
            create_vs_makefile_vcxproj(context, &package.id, target)?;
        }
    }
    write!(o, "\tEndGlobalSection\r\n")?;

    write!(o, "\tGlobalSection(SolutionProperties) = preSolution\r\n")?;
    write!(o, "\t\tHideSolutionNode = FALSE\r\n")?;
    write!(o, "\tEndGlobalSection\r\n")?;

    write!(o, "\tGlobalSection(ExtensibilityGlobals) = postSolution\r\n")?;
    write!(o, "\t\tSolutionGuid = {{{}}}\r\n", uuid_sln.to_hyphenated_ref().encode_upper(&mut uuid_buf))?;
    write!(o, "\tEndGlobalSection\r\n")?;

    write!(o, "EndGlobal\r\n")?;

    Ok(())
}

fn create_vs_makefile_vcxproj(context: &Context, package: &metadata::PackageId, bin: &metadata::PackageTarget) -> io::Result<()> {
    let path = context.vs.join("vs2017").join(&format!("{}.vcxproj", bin.name));
    let mut o = File::create(&path)?;
    let mut uuid_buf = [b'!'; 40];

    write!(o, "<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n")?;
    write!(o, "<Project DefaultTargets=\"Build\" ToolsVersion=\"15.0\" xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">\r\n")?;
    write!(o, "  <ItemGroup Label=\"ProjectConfigurations\">\r\n")?;
    for arch in ARCHES.iter().copied() {
        for config in CONFIGS.iter().copied() {
            write!(o,
                concat!(
                    "    <ProjectConfiguration Include=\"{config}|{arch}\">\r\n",
                    "      <Configuration>{config}</Configuration>\r\n",
                    "      <Platform>{arch}</Platform>\r\n",
                    "    </ProjectConfiguration>\r\n",
                ),
                config=config.vs_name,
                arch=arch.vs_proj_name,
            )?;
        }
    }
    write!(o, "  </ItemGroup>\r\n")?;
    write!(o, "  <PropertyGroup Label=\"Globals\">\r\n")?;
    write!(o, "    <VCProjectVersion>15.0</VCProjectVersion>\r\n")?;
    write!(o, "    <ProjectGuid>{{{}}}</ProjectGuid>\r\n", project_guid(context, package, bin).to_hyphenated_ref().encode_upper(&mut uuid_buf))?;
    write!(o, "    <Keyword>Win32Proj</Keyword>\r\n")?;
    write!(o, "  </PropertyGroup>\r\n")?;
    write!(o, "  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.Default.props\" />\r\n")?;
    for arch in ARCHES.iter().copied() {
        for config in CONFIGS.iter().copied() {
            write!(o, "  <PropertyGroup Condition=\"'$(Configuration)|$(Platform)'=='{config}|{arch}'\" Label=\"Configuration\">\r\n", config=config.vs_name, arch=arch.vs_proj_name)?;
            write!(o, "    <ConfigurationType>Makefile</ConfigurationType>\r\n")?;
            write!(o, "    <UseDebugLibraries>{}</UseDebugLibraries>\r\n", config.vs_name == "Debug")?;
            write!(o, "    <PlatformToolset>v141</PlatformToolset>\r\n")?;
            write!(o, "  </PropertyGroup>\r\n")?;
        }
    }
    write!(o, "  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.props\" />\r\n")?;
    write!(o, "  <ImportGroup Label=\"ExtensionSettings\">\r\n")?;
    write!(o, "  </ImportGroup>\r\n")?;
    write!(o, "  <ImportGroup Label=\"Shared\">\r\n")?;
    write!(o, "  </ImportGroup>\r\n")?;
    for arch in ARCHES.iter().copied() {
        for config in CONFIGS.iter().copied() {
            write!(o, "  <ImportGroup Label=\"PropertySheets\" Condition=\"'$(Configuration)|$(Platform)'=='{config}|{arch}'\">\r\n", config=config.vs_name, arch=arch.vs_proj_name)?;
            write!(o, "    <Import Project=\"$(UserRootDir)\\Microsoft.Cpp.$(Platform).user.props\" Condition=\"exists('$(UserRootDir)\\Microsoft.Cpp.$(Platform).user.props')\" Label=\"LocalAppDataPlatform\" />\r\n")?;
            write!(o, "  </ImportGroup>\r\n")?;
        }
    }
    write!(o, "    <PropertyGroup Label=\"UserMacros\" />\r\n")?;
    for arch in ARCHES.iter().copied() {
        for config in CONFIGS.iter().copied() {
            let defines_arch    = if arch.vs_proj_name == "Win32" { "_WIN32;" } else { "" };
            let defines_config  = if config.vs_name == "Debug" { "_DEBUG;" } else { "NDEBUG;" };
            write!(o, "  <PropertyGroup Condition=\"'$(Configuration)|$(Platform)'=='{config}|{arch}'\">\r\n", config=config.vs_name, arch=arch.vs_proj_name)?;
            write!(o, "    <NMakeBuildCommandLine>cargo +stable-{arch}-pc-windows-msvc build --target {arch}-pc-windows-msvc{flags}</NMakeBuildCommandLine>\r\n", arch=arch.cargo_name, flags=config.cargo_build_flags)?;
            write!(o, "    <NMakeOutput>$(SolutionDir)..\\target\\{arch}-pc-windows-msvc\\{config}\\{bin_name}.exe</NMakeOutput>\r\n", arch=arch.cargo_name, config=config.target_dir, bin_name=bin.name)?;
            write!(o, "    <NMakeCleanCommandLine>cargo clean</NMakeCleanCommandLine>\r\n")?;
            write!(o, "    <NMakeReBuildCommandLine>cargo clean &amp;&amp; cargo +stable-{} build{}</NMakeReBuildCommandLine>\r\n", arch.cargo_name, config.cargo_build_flags)?;
            write!(o, "    <NMakePreprocessorDefinitions>{}{}$(NMakePreprocessorDefinitions)</NMakePreprocessorDefinitions>\r\n", defines_arch, defines_config)?;
            write!(o, "    <OutDir>$(SolutionDir)..\\target\\{arch}-pc-windows-msvc\\{config}\\</OutDir>", arch=arch.cargo_name, config=config.target_dir)?;
            write!(o, "    <IntDir>$(SolutionDir)int\\{arch}-pc-windows-msvc\\{config}\\</IntDir>", arch=arch.cargo_name, config=config.target_dir)?; // Putting IntDir inside target would cause VS to choke on clean
            write!(o, "    <LocalDebuggerWorkingDirectory>$(SolutionDir)..\\</LocalDebuggerWorkingDirectory>")?; // use workspace dir
            write!(o, "  </PropertyGroup>\r\n")?;
        }
    }
    write!(o, "  <ItemDefinitionGroup>\r\n")?;
    write!(o, "  </ItemDefinitionGroup>\r\n")?;
    write!(o, "  <ItemGroup>\r\n")?;
    write!(o, "  </ItemGroup>\r\n")?;
    write!(o, "  <Import Project=\"$(VCTargetsPath)\\Microsoft.Cpp.targets\" />\r\n")?;
    write!(o, "  <ImportGroup Label=\"ExtensionTargets\">\r\n")?;
    write!(o, "  </ImportGroup>\r\n")?;
    write!(o, "</Project>")?;
    Ok(())
}

fn project_guid(context: &Context, package: &metadata::PackageId, bin: &metadata::PackageTarget) -> Uuid {
    let name = format!("{}|{}|{}", context.vs.display(), package, bin.name);
    Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes())
}
