$ErrorActionPreference = 'Stop'

if ($Args.Length -eq 1) {
  $Version = $Args.Get(0)
}

$BinDir = "${Home}\.nitrogen\bin"

$NitrogenTar = "$BinDir\nitrogen.tar.gz"
$NitrogenBin = "$BinDir\nitrogen.exe"
$Target = 'x86_64-pc-windows-msvc'

$DownloadUrl = if (!$Version) {
  $RedirectUrl = curl.exe -s -L -I -o nul -w '%{url_effective}' "https://github.com/capeprivacy/nitrogen/releases/latest/download"

  $Splits = $RedirectUrl.Split("/")
  $Version = $Splits.Get($Splits.Length - 1)

  "https://github.com/capeprivacy/nitrogen/releases/latest/download/nitrogen_${Version}_${Target}.zip"
} else {
  "https://github.com/capeprivacy/nitrogen/releases/download/${Version}/nitrogen_${Version}_${Target}.zip"
}

if (!(Test-Path $BinDir)) {
  New-Item $BinDir -ItemType Directory | Out-Null
}

curl.exe -Lo $NitrogenTar $DownloadUrl

tar.exe xf $NitrogenTar -C $BinDir

Remove-Item $NitrogenTar

$User = [System.EnvironmentVariableTarget]::User
$Path = [System.Environment]::GetEnvironmentVariable('Path', $User)
if (!(";${Path};".ToLower() -like "*;${BinDir};*".ToLower())) {
  [System.Environment]::SetEnvironmentVariable('Path', "${Path};${BinDir}", $User)
  $Env:Path += ";${BinDir}"
}

Write-Output "Nitrogen was installed successfully to ${NitrogenBin}"
Write-Output "Run 'nitrogen --help' to get started"
