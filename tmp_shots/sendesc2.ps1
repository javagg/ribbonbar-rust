Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Kbd {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] inputs, int size);
  [StructLayout(LayoutKind.Sequential)]
  public struct INPUT { public uint type; public InputUnion u; }
  [StructLayout(LayoutKind.Explicit)]
  public struct InputUnion { [FieldOffset(0)] public KEYBDINPUT ki; [FieldOffset(0)] public MOUSEINPUT mi; }
  [StructLayout(LayoutKind.Sequential)]
  public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }
  [StructLayout(LayoutKind.Sequential)]
  public struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr dwExtraInfo; }
  public static void Tap(ushort vk) {
    var downs = new INPUT[1]; downs[0].type = 1; downs[0].u.ki.wVk = vk;
    var ups = new INPUT[1]; ups[0].type = 1; ups[0].u.ki.wVk = vk; ups[0].u.ki.dwFlags = 2;
    SendInput(1, downs, Marshal.SizeOf(typeof(INPUT)));
    SendInput(1, ups, Marshal.SizeOf(typeof(INPUT)));
  }
}
"@
$proc = Get-Process cad_demo | Select-Object -First 1
$ok = [Kbd]::SetForegroundWindow($proc.MainWindowHandle)
Start-Sleep -Milliseconds 500
$fg = [Kbd]::GetForegroundWindow()
Write-Output "setfg=$ok foreground_is_cad=$($fg -eq $proc.MainWindowHandle)"
[Kbd]::Tap(0x1B)  # VK_ESCAPE
Write-Output "esc tapped"
