<#
  Maestro interactive "wait mode" learning (Windows / WinMM).

  Shows the next note of a song; you play it on your keyboard; it only advances
  when you hit the right note, echoes it back, and scores your accuracy at the
  end. This is the same engine as `maestro learn` in the Rust app, usable on a
  real CASIO without building the `midi` feature.

  Examples (from the repo root):
    .\scripts\windows\maestro-learn.ps1 -Id twinkle
    .\scripts\windows\maestro-learn.ps1 -Id el_manicero -OctaveAny
    .\scripts\windows\maestro-learn.ps1 -File examples\ode_to_joy.txt

  -OctaveAny : accept the right note in any octave (forgiving).
  -NoEcho    : don't play the note back through the keyboard.
#>
param(
  [string]$Id = "",
  [string]$File = "",
  [ValidateSet("songs","scales")]
  [string]$Kind = "songs",
  [int]$InDevice = -1,
  [int]$OutDevice = -1,
  [string]$DataDir = "",
  [switch]$OctaveAny,
  [switch]$NoEcho
)

. "$PSScriptRoot\midi-common.ps1"

# --- resolve the target notes -------------------------------------------------
function Load-Notes {
  if ($File) {
    if (-not (Test-Path $File)) { throw "File not found: $File" }
    # Minimal text-tab reader (NOTE:DUR tokens); mirrors the Rust importer.
    $pc = @{ "C"=0;"D"=2;"E"=4;"F"=5;"G"=7;"A"=9;"B"=11 }
    $notes = @()
    foreach ($line in Get-Content $File) {
      $l = $line.Trim()
      if ($l -eq "" -or $l.StartsWith("#")) { continue }
      foreach ($tok in ($l -split "\s+")) {
        $tok = $tok.Trim("|"); if ($tok -eq "") { continue }
        $name = ($tok -split ":")[0]
        if ($name -match "^[Rr]$") { continue }
        if ($name -match "^([A-Ga-g])([#b]?)(-?\d+)$") {
          $v = $pc[$matches[1].ToUpper()]
          if ($matches[2] -eq "#") { $v++ } elseif ($matches[2] -eq "b") { $v-- }
          $midi = ([int]$matches[3] + 1) * 12 + $v
          if ($midi -ge 0 -and $midi -le 127) { $notes += [int]$midi }
        }
      }
    }
    return ,$notes
  }
  $root = Resolve-DataDir $DataDir
  $f = Join-Path $root "$Kind\$Id.json"
  if (-not (Test-Path $f)) { throw "Not found: $f" }
  $data = Get-Content $f -Raw | ConvertFrom-Json
  if ($Kind -eq "scales") { return ,([int[]]$data.notes) }
  return ,([int[]]($data.notes | Where-Object { [int]$_[1] -gt 0 } | ForEach-Object { [int]$_[0] }))
}

if (-not $Id -and -not $File) { Write-Error "Provide -Id <song> or -File <tab>"; return }
$expected = Load-Notes
if ($expected.Count -eq 0) { Write-Error "No playable notes found."; return }

# --- open devices -------------------------------------------------------------
$ins = Get-MidiInDevices
if ($ins.Count -eq 0) { Write-Error "No MIDI input devices. Is your keyboard connected?"; return }
if ($InDevice -lt 0) { $InDevice = Find-CasioIn $ins }
$outs = Get-MidiOutDevices
if ($OutDevice -lt 0) { $OutDevice = Find-CasioOut $outs }

$listener = New-Object MaestroMidi+Listener
$rc = $listener.Open($InDevice)
if ($rc -ne 0) { Write-Error "Could not open MIDI input (rc=$rc)."; return }
$out = $null
if (-not $NoEcho) { try { $out = Open-MidiOut $OutDevice } catch { $out = $null } }

Write-Host ("Input : device {0} ({1})" -f $InDevice, ($ins | ? {$_.Index -eq $InDevice}).Name)
Write-Host ("Learning {0} notes. Play each highlighted note. Ctrl+C to stop." -f $expected.Count)
if ($OctaveAny) { Write-Host "(octave-forgiving mode on)" }
Write-Host ""

# --- the wait-mode loop -------------------------------------------------------
$idx = 0; $hits = 0; $misses = 0
try {
  while ($idx -lt $expected.Count) {
    $target = [int]$expected[$idx]
    Write-Host ("`r-> play {0}   ({1} to go)        " -f (Note-Name $target), ($expected.Count - $idx)) -NoNewline
    $played = $null
    while ($played -eq $null) {
      $n = 0
      if ($listener.Notes.TryDequeue([ref]$n)) { $played = [int]$n } else { Start-Sleep -Milliseconds 8 }
    }
    $ok = if ($OctaveAny) { ($played % 12) -eq ($target % 12) } else { $played -eq $target }
    if ($ok) {
      $hits++; $idx++
      if ($out) { Send-NoteOn $out $target 90; Start-Sleep -Milliseconds 140; Send-NoteOff $out $target }
      Write-Host ("`r   OK  {0}                         " -f (Note-Name $target))
    } else {
      $misses++
      Write-Host ("`r   x  you played {0}, need {1}     " -f (Note-Name $played), (Note-Name $target))
    }
  }
  $total = $hits + $misses
  $acc = if ($total -gt 0) { [math]::Round(100.0 * $hits / $total) } else { 100 }
  Write-Host ""
  Write-Host ("Done! {0}/{1} notes, {2} misses, {3}% accuracy." -f $hits, $expected.Count, $misses, $acc)
}
finally {
  $listener.Close()
  if ($out) { Close-MidiOut $out }
}
