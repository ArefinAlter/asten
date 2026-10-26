# Drives a native Windows dialog by UI Automation, for the acceptance runs.
#
#   dialog.ps1 -Title "Restore the library" -Button "Restore and restart"
#   dialog.ps1 -Title "Save" -Path "D:\scratch\out.asten" -Button "Save"
#
# Finds a top-level window whose title matches, optionally types a path into
# its file-name box, and presses the named button. Prints what it did, one line
# of JSON. Exits 2 if no such window appears in time.
param(
  [Parameter(Mandatory = $true)][string]$Title,
  [string]$Path = "",
  [string]$Button = "",
  [int]$TimeoutSec = 20,
  [string]$Keys = ""
)
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$root = $A::RootElement
$deadline = (Get-Date).AddSeconds($TimeoutSec)
$window = $null
while ((Get-Date) -lt $deadline -and -not $window) {
  $top = $root.FindAll([System.Windows.Automation.TreeScope]::Children, [System.Windows.Automation.Condition]::TrueCondition)
  foreach ($w in $top) {
    $name = $w.Current.Name
    if ($name -and $name -like "*$Title*") { $window = $w; break }
    # A dialog is owned by the window that opened it; UI Automation lists it under that window.
    $owned = $w.FindAll([System.Windows.Automation.TreeScope]::Children,
      (New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::Window)))
    foreach ($o in $owned) {
      $oname = $o.Current.Name
      if ($oname -and $oname -like "*$Title*") { $window = $o; break }
    }
    if ($window) { break }
  }
  if (-not $window) { Start-Sleep -Milliseconds 300 }
}
if (-not $window) {
  Write-Output ('{"found":false,"title":"' + $Title + '"}')
  exit 2
}
Start-Sleep -Milliseconds 400
$typed = $false
if ($Path) {
  $edits = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants,
    (New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::Edit)))
  # The file-name box: named "File name:" (automation id 1001), not the search or address box.
  $target = $null
  foreach ($e in $edits) {
    if ($e.Current.AutomationId -eq "1001" -or $e.Current.Name -like "File name*") { $target = $e; break }
  }
  if ($target) {
    try {
      $vp = $target.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
      $vp.SetValue($Path)
      $typed = $true
    } catch {
      # Fall back to typing into the focused box.
      Add-Type -AssemblyName System.Windows.Forms
      $target.SetFocus()
      Start-Sleep -Milliseconds 300
      [System.Windows.Forms.SendKeys]::SendWait("^a")
      $escaped = $Path -replace '([+^%~(){}\[\]])', '{$1}'
      [System.Windows.Forms.SendKeys]::SendWait($escaped)
      $typed = $true
    }
  }
}
if ($Path -and -not $typed) {
  # The Explorer-style dialog hides its file-name box from UI Automation; it has
  # the keyboard focus when it opens, so type the path as a person would.
  Add-Type -AssemblyName System.Windows.Forms
  Add-Type -Namespace Win -Name User32 -MemberDefinition @"
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool SetForegroundWindow(System.IntPtr h);
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, System.UIntPtr extra);
"@
  # A background process may not take the foreground; a tap on Alt lifts that lock.
  [Win.User32]::keybd_event(0x12, 0, 0, [System.UIntPtr]::Zero)
  [Win.User32]::keybd_event(0x12, 0, 2, [System.UIntPtr]::Zero)
  $h = [System.IntPtr]$window.Current.NativeWindowHandle
  [void][Win.User32]::SetForegroundWindow($h)
  try { $window.SetFocus() } catch {}
  Start-Sleep -Milliseconds 700
  $escaped = $Path -replace '([+^%~(){}\[\]])', '{$1}'
  [System.Windows.Forms.SendKeys]::SendWait($escaped)
  Start-Sleep -Milliseconds 300
  [System.Windows.Forms.SendKeys]::SendWait("{ENTER}")
  Write-Output ('{"found":true,"window":"' + ($window.Current.Name -replace '"', "'") + '","typed":true,"pressed":"Enter (keyboard)"}')
  exit 0
}
function Focus-Window($w) {
  Add-Type -AssemblyName System.Windows.Forms
  Add-Type -Namespace Win -Name User32b -MemberDefinition @"
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool SetForegroundWindow(System.IntPtr h);
[System.Runtime.InteropServices.DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, System.UIntPtr extra);
"@
  [Win.User32b]::keybd_event(0x12, 0, 0, [System.UIntPtr]::Zero)
  [Win.User32b]::keybd_event(0x12, 0, 2, [System.UIntPtr]::Zero)
  [void][Win.User32b]::SetForegroundWindow([System.IntPtr]$w.Current.NativeWindowHandle)
  try { $w.SetFocus() } catch {}
  Start-Sleep -Milliseconds 700
}
if ($Keys) {
  # Keys by name: the dialog's own buttons are not exposed to UI Automation.
  Focus-Window $window
  [System.Windows.Forms.SendKeys]::SendWait($Keys)
  Write-Output ('{"found":true,"window":"' + ($window.Current.Name -replace '"', "'") + '","pressed":"keys ' + $Keys + '"}')
  exit 0
}
$buttons = $window.FindAll([System.Windows.Automation.TreeScope]::Descendants,
  (New-Object System.Windows.Automation.PropertyCondition($A::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)))
$pressed = $null
foreach ($b in $buttons) {
  $n = $b.Current.Name -replace '&', ''
  if ($n -eq $Button -or $n -like "$Button*") {
    $ip = $b.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    $ip.Invoke()
    $pressed = $n
    break
  }
}
$names = ($buttons | ForEach-Object { $_.Current.Name }) -join "|"
Write-Output ('{"found":true,"window":"' + ($window.Current.Name -replace '"', "'") + '","typed":' + ($typed.ToString().ToLower()) + ',"pressed":"' + $pressed + '","buttons":"' + ($names -replace '"', "'") + '"}')
if (-not $pressed) { exit 3 }
