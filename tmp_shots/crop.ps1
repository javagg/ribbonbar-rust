param([string]$In, [string]$Out, [int]$X, [int]$Y, [int]$W, [int]$H, [int]$Zoom = 2)
Add-Type -AssemblyName System.Drawing
$src = [System.Drawing.Image]::FromFile($In)
$crop = New-Object System.Drawing.Bitmap(($W*$Zoom), ($H*$Zoom))
$g = [System.Drawing.Graphics]::FromImage($crop)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
$dst = New-Object System.Drawing.Rectangle(0, 0, ($W*$Zoom), ($H*$Zoom))
$srcRect = New-Object System.Drawing.Rectangle($X, $Y, $W, $H)
$g.DrawImage($src, $dst, $srcRect, [System.Drawing.GraphicsUnit]::Pixel)
$crop.Save($Out)
$g.Dispose(); $crop.Dispose(); $src.Dispose()
Write-Output "ok $Out"
