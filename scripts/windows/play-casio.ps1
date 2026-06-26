<#
  Maestro -> MIDI player (Windows / WinMM).
  Plays a Maestro catalogue JSON (song, scale, or chord) to a MIDI output
  device, defaulting to a connected CASIO. Use this to hear the catalogue on a
  real keyboard when you can't build the Rust binary with the `midi` feature.

  Examples (from the repo root or scripts\windows):
    .\scripts\windows\play-casio.ps1 -List
    .\scripts\windows\play-casio.ps1 -Id fur_elise
    .\scripts\windows\play-casio.ps1 -Id el_manicero
    .\scripts\windows\play-casio.ps1 -Id g_major -Kind scales
    .\scripts\windows\play-casio.ps1 -Id c_i_iv_v -Kind chords
    .\scripts\windows\play-casio.ps1 -Id twinkle -Device 3
#>
param(
  [string]$Id = "twinkle",
  [ValidateSet("songs","scales","chords")]
  [string]$Kind = "songs",
  [int]$Device = -1,
  [string]$DataDir = "",
  [switch]$List
)

. "$PSScriptRoot\midi-common.ps1"

$devices = Get-MidiOutDevices
if ($List) {
  Write-Host "MIDI output devices:"
  $devices | ForEach-Object { "{0,2}: {1}" -f $_.Index, $_.Name } | Write-Host
  return
}

if ($Device -lt 0) { $Device = (Find-CasioOut $devices) }
$devName = ($devices | Where-Object { $_.Index -eq $Device }).Name

$root = Resolve-DataDir $DataDir
$file = Join-Path $root "$Kind\$Id.json"
if (-not (Test-Path $file)) { Write-Error "Not found: $file"; return }
$data = Get-Content $file -Raw | ConvertFrom-Json

Write-Host ("Playing '{0}' ({1}) -> device {2}: {3}" -f $data.name, $Kind, $Device, $devName)
$out = Open-MidiOut $Device
try {
  if ($Kind -eq "songs") {
    foreach ($ev in $data.notes) {
      $n=[int]$ev[0]; $v=[int]$ev[1]; $d=[int]$ev[2]
      if ($v -gt 0) { Send-NoteOn $out $n $v }
      Start-Sleep -Milliseconds $d
      if ($v -gt 0) { Send-NoteOff $out $n }
    }
  } elseif ($Kind -eq "scales") {
    foreach ($n in $data.notes) { Send-NoteOn $out ([int]$n) 80; Start-Sleep -Milliseconds 400; Send-NoteOff $out ([int]$n) }
  } elseif ($Kind -eq "chords") {
    foreach ($chord in $data.chords) {
      foreach ($n in $chord) { Send-NoteOn $out ([int]$n) 80 }
      Start-Sleep -Milliseconds 700
      foreach ($n in $chord) { Send-NoteOff $out ([int]$n) }
      Start-Sleep -Milliseconds 120
    }
  }
} finally { Close-MidiOut $out; Write-Host "Done." }
