param([string]$OutPath = "D:\srcs\gpapp\tmp_shots\win.png")
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32Cap {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
  public struct RECT { public int Left, Top, Right, Bottom; }
}
"@
[Win32Cap]::SetProcessDPIAware() | Out-Null
$proc = Get-Process cad_demo -ErrorAction Stop | Select-Object -First 1
$hwnd = $proc.MainWindowHandle
$rect = New-Object Win32Cap+RECT
[Win32Cap]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
$w = $rect.Right - $rect.Left; $h = $rect.Bottom - $rect.Top
$bmp = New-Object System.Drawing.Bitmap($w, $h)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $g.GetHdc()
[Win32Cap]::PrintWindow($hwnd, $hdc, 2) | Out-Null
$g.ReleaseHdc($hdc); $g.Dispose()
$bmp.Save($OutPath); $bmp.Dispose()
Write-Output "saved $w x $h rect=($($rect.Left),$($rect.Top)) -> $OutPath"
