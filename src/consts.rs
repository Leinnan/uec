#[cfg(target_os = "windows")]
pub const PLATFORM: &str = "Win64";
#[cfg(target_os = "linux")]
pub const PLATFORM: &str = "Linux";
#[cfg(target_os = "macos")]
pub const PLATFORM: &str = "Mac";

#[cfg(target_os = "windows")]
pub const EDITOR: &str = "Engine/Binaries/Win64/UnrealEditor.exe";
#[cfg(target_os = "macos")]
pub const EDITOR: &str = "Engine/Binaries/Mac/UnrealEditor.app/Contents/MacOS/UnrealEditor";
#[cfg(target_os = "linux")]
pub const EDITOR: &str = "Engine/Binaries/Linux/UnrealEditor";

#[cfg(target_os = "windows")]
pub const BUILD_SCRIPT: &str = "Engine/Build/BatchFiles/Build.bat";
#[cfg(target_os = "macos")]
pub const BUILD_SCRIPT: &str = "Engine/Build/BatchFiles/Mac/Build.sh";
#[cfg(target_os = "linux")]
pub const BUILD_SCRIPT: &str = "Engine/Build/BatchFiles/Linux/Build.sh";

#[cfg(target_os = "windows")]
pub const GENERATE_PROJ_SCRIPT: &str = "Engine/Build/BatchFiles/GenerateProjectFiles.bat";
#[cfg(target_os = "macos")]
pub const GENERATE_PROJ_SCRIPT: &str = "Engine/Build/BatchFiles/Mac/GenerateProjectFiles.sh";
#[cfg(target_os = "linux")]
pub const GENERATE_PROJ_SCRIPT: &str = "Engine/Build/BatchFiles/Linux/GenerateProjectFiles.sh";

#[cfg(target_os = "windows")]
pub const UAT_SCRIPT: &str = "Engine/Build/BatchFiles/RunUAT.bat";
#[cfg(not(target_os = "windows"))]
pub const UAT_SCRIPT: &str = "Engine/Build/BatchFiles/RunUAT.sh";

#[cfg(target_os = "windows")]
pub const DOTNET: &str = "Engine/Binaries/ThirdParty/DotNet/6.0.302/windows/dotnet.exe";

#[cfg(target_os = "windows")]
pub const BUILD_TOOL: &str = "Engine/Binaries/DotNET/UnrealBuildTool/UnrealBuildTool.dll";
