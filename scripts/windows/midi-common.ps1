<#
  Shared WinMM MIDI helpers for the Maestro Windows scripts.
  Dot-source this file:  . "$PSScriptRoot\midi-common.ps1"
#>

Add-Type -ErrorAction SilentlyContinue -TypeDefinition @'
using System;
using System.Collections.Concurrent;
using System.Runtime.InteropServices;

public static class MaestroMidi {
  // ----- output -----
  [DllImport("winmm.dll")] public static extern uint midiOutGetNumDevs();
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Auto)]
  public struct OUTCAPS {
    public ushort wMid; public ushort wPid; public uint vDriverVersion;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string szPname;
    public ushort wTechnology; public ushort wVoices; public ushort wNotes;
    public ushort wChannelMask; public uint dwSupport;
  }
  [DllImport("winmm.dll", CharSet=CharSet.Auto)] public static extern uint midiOutGetDevCaps(UIntPtr id, ref OUTCAPS c, uint size);
  [DllImport("winmm.dll")] public static extern uint midiOutOpen(out IntPtr h, uint id, IntPtr cb, IntPtr inst, uint flags);
  [DllImport("winmm.dll")] public static extern uint midiOutShortMsg(IntPtr h, uint msg);
  [DllImport("winmm.dll")] public static extern uint midiOutClose(IntPtr h);

  // ----- input -----
  [DllImport("winmm.dll")] public static extern uint midiInGetNumDevs();
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Auto)]
  public struct INCAPS {
    public ushort wMid; public ushort wPid; public uint vDriverVersion;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string szPname;
    public uint dwSupport;
  }
  [DllImport("winmm.dll", CharSet=CharSet.Auto)] public static extern uint midiInGetDevCaps(UIntPtr id, ref INCAPS c, uint size);

  public delegate void MidiInProc(IntPtr h, uint msg, IntPtr inst, IntPtr p1, IntPtr p2);
  [DllImport("winmm.dll")] public static extern uint midiInOpen(out IntPtr h, uint id, MidiInProc cb, IntPtr inst, uint flags);
  [DllImport("winmm.dll")] public static extern uint midiInStart(IntPtr h);
  [DllImport("winmm.dll")] public static extern uint midiInStop(IntPtr h);
  [DllImport("winmm.dll")] public static extern uint midiInClose(IntPtr h);

  // A tiny note-on listener that survives across PowerShell pipeline calls.
  public class Listener {
    public ConcurrentQueue<int> Notes = new ConcurrentQueue<int>();
    private IntPtr _h = IntPtr.Zero;
    private MidiInProc _proc; // keep the delegate alive
    public uint Open(int device) {
      _proc = new MidiInProc(Callback);
      uint rc = midiInOpen(out _h, (uint)device, _proc, IntPtr.Zero, 0x30000 /*CALLBACK_FUNCTION*/);
      if (rc == 0) midiInStart(_h);
      return rc;
    }
    private void Callback(IntPtr h, uint msg, IntPtr inst, IntPtr p1, IntPtr p2) {
      if (msg == 0x3C3 /*MIM_DATA*/) {
        uint dw = (uint)p1.ToInt64();
        uint status = dw & 0xFF, data1 = (dw >> 8) & 0xFF, data2 = (dw >> 16) & 0xFF;
        if ((status & 0xF0) == 0x90 && data2 > 0) Notes.Enqueue((int)data1);
      }
    }
    public void Close() {
      if (_h != IntPtr.Zero) { midiInStop(_h); midiInClose(_h); _h = IntPtr.Zero; }
    }
  }
}
'@

function Get-MidiOutDevices {
  $n = [MaestroMidi]::midiOutGetNumDevs(); $out = @()
  for ($i=0; $i -lt $n; $i++) {
    $c = New-Object MaestroMidi+OUTCAPS
    [void][MaestroMidi]::midiOutGetDevCaps([UIntPtr]::new([uint64]$i), [ref]$c, [uint32][Runtime.InteropServices.Marshal]::SizeOf($c))
    $out += [pscustomobject]@{ Index=$i; Name=$c.szPname }
  }
  return $out
}

function Get-MidiInDevices {
  $n = [MaestroMidi]::midiInGetNumDevs(); $out = @()
  for ($i=0; $i -lt $n; $i++) {
    $c = New-Object MaestroMidi+INCAPS
    [void][MaestroMidi]::midiInGetDevCaps([UIntPtr]::new([uint64]$i), [ref]$c, [uint32][Runtime.InteropServices.Marshal]::SizeOf($c))
    $out += [pscustomobject]@{ Index=$i; Name=$c.szPname }
  }
  return $out
}

function Find-CasioOut($devices) {
  $c = $devices | Where-Object { $_.Name -match "CASIO" } | Select-Object -First 1
  if ($c) { return [int]$c.Index } else { return 0 }
}
function Find-CasioIn($devices) {
  $c = $devices | Where-Object { $_.Name -match "CASIO" } | Select-Object -First 1
  if ($c) { return [int]$c.Index } else { return 0 }
}

function Open-MidiOut([int]$device) {
  $h = [IntPtr]::Zero
  $rc = [MaestroMidi]::midiOutOpen([ref]$h, [uint32]$device, [IntPtr]::Zero, [IntPtr]::Zero, 0)
  if ($rc -ne 0) { throw "midiOutOpen failed (rc=$rc)" }
  return $h
}
function Send-NoteOn([IntPtr]$h,[int]$n,[int]$v) { [void][MaestroMidi]::midiOutShortMsg($h, [uint32](0x90 -bor ($n -shl 8) -bor ($v -shl 16))) }
function Send-NoteOff([IntPtr]$h,[int]$n)        { [void][MaestroMidi]::midiOutShortMsg($h, [uint32](0x80 -bor ($n -shl 8))) }
function Close-MidiOut([IntPtr]$h) { [void][MaestroMidi]::midiOutClose($h) }

function Resolve-DataDir([string]$override) {
  if ($override) { return $override }
  if ($env:MAESTRO_DATA_DIR) { return $env:MAESTRO_DATA_DIR }
  $dir = $PSScriptRoot
  for ($i=0; $i -lt 5; $i++) {
    $cand = Join-Path $dir "data"
    if (Test-Path (Join-Path $cand "songs")) { return $cand }
    $dir = Split-Path -Parent $dir
    if (-not $dir) { break }
  }
  return (Join-Path $PSScriptRoot "data")
}

function Note-Name([int]$n) {
  $names = @("C","C#","D","D#","E","F","F#","G","G#","A","A#","B")
  return ("{0}{1}" -f $names[$n % 12], [int]([math]::Floor($n/12) - 1))
}
