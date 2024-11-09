$ErrorActionPreference = 'Stop'

$version = $args[0] -replace '.*/', "$1"

$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest https://aka.ms/wingetcreate/latest -OutFile wingetcreate.exe
.\wingetcreate.exe update --urls "https://github.com/Leinnan/uec/releases/download/${version}/uec-${version}-x86_64.msi" --version "${version}" --submit --token $args[1] "MevLyshkin.uec"
