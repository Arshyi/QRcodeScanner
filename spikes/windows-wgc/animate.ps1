Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$form = [Windows.Forms.Form]::new()
$form.Text = "QRForge WGC benchmark activity"
$form.Size = [Drawing.Size]::new(900, 650)
$form.StartPosition = "CenterScreen"
$form.TopMost = $true
$form.FormBorderStyle = "FixedToolWindow"

$label = [Windows.Forms.Label]::new()
$label.Dock = "Fill"
$label.TextAlign = "MiddleCenter"
$label.Font = [Drawing.Font]::new("Segoe UI", 32)
$form.Controls.Add($label)

$frame = 0
$timer = [Windows.Forms.Timer]::new()
$timer.Interval = 16
$timer.Add_Tick({
  $script:frame++
  $label.Text = "QRForge capture frame $script:frame"
  $red = ($script:frame * 7) % 200 + 30
  $green = ($script:frame * 11) % 200 + 30
  $blue = ($script:frame * 13) % 200 + 30
  $form.BackColor = [Drawing.Color]::FromArgb($red, $green, $blue)
  $label.BackColor = $form.BackColor
})
$timer.Start()
[Windows.Forms.Application]::Run($form)

