# =======================
#   Benchmark Config
# =======================
$baseUrl   = "http://127.0.0.1:8080"
$serverPid = 24032     # Python or Rust server PID
$bulkCount = 50

# Function: Get memory of process
function Get-MemMB($procId) {
    try {
        $p = Get-Process -Id $procId -ErrorAction Stop
        return [math]::Round(($p.WorkingSet / 1MB), 2)
    } catch {
        return 0
    }
}

# Function: Track peak memory
function Get-PeakMem($procId) {
    try {
        $p = Get-Process -Id $procId -ErrorAction Stop
    } catch {
        return 0
    }

    $peak = 0
    while ($true) {
        try {
            $p.Refresh()
            $cur = $p.WorkingSet / 1MB
            $peak = [math]::Max($peak, $cur)
            Start-Sleep -Milliseconds 50
        } catch {
            break
        }
    }
    return [math]::Round($peak, 2)
}

# Measure single request
function Measure-Request($method, $url, $body, $procId) {

    $memBefore = Get-MemMB $procId
    $timer = [System.Diagnostics.Stopwatch]::StartNew()

    try {
        if ($body -ne $null) {
            $resp = Invoke-WebRequest -Method $method -Uri $url -Body $body -TimeoutSec 20
        } else {
            $resp = Invoke-WebRequest -Method $method -Uri $url -TimeoutSec 20
        }
    } catch {
        return @{ error = $_.Exception.Message }
    }

    $timer.Stop()
    $memAfter = Get-MemMB $procId

    return @{
        latency_ms  = $timer.Elapsed.TotalMilliseconds
        memory_mb   = $memAfter
        status      = $resp.StatusCode
    }
}

# -----------------------------
#   BULK OPERATIONS
# -----------------------------

function Bulk-Create($url, $count, $procId) {

    $items = @(for ($i=0; $i -lt $count; $i++) { @{ title = "bulk-$i" } })
    $json = $items | ConvertTo-Json

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    $resp = Invoke-WebRequest -Method POST -Uri "$url/bulk_create" `
                -Body $json -ContentType "application/json"
    $timer.Stop()

    return @{
        latency_ms = $timer.Elapsed.TotalMilliseconds
        memory_mb  = Get-MemMB $procId
        created    = ($resp.Content | ConvertFrom-Json).created_count
    }
}

function Bulk-Update($url, $ids, $procId) {

    $payload = @()
    foreach ($id in $ids) {
        $payload += @{ id = $id; title = "updated-$id"; completed = $true }
    }

    $json = $payload | ConvertTo-Json

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    $resp = Invoke-WebRequest -Method PUT -Uri "$url/bulk_update" `
                -Body $json -ContentType "application/json"
    $timer.Stop()

    return @{
        latency_ms = $timer.Elapsed.TotalMilliseconds
        memory_mb  = Get-MemMB $procId
        updated    = ($resp.Content | ConvertFrom-Json).updated
    }
}

function Bulk-Delete($url, $ids, $procId) {

    $json = $ids | ConvertTo-Json

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    $resp = Invoke-WebRequest -Method DELETE `
            -Uri "$url/bulk_delete" `
            -Body $json -ContentType "application/json"
    $timer.Stop()

    return @{
        latency_ms = $timer.Elapsed.TotalMilliseconds
        memory_mb  = Get-MemMB $procId
        deleted    = ($resp.Content | ConvertFrom-Json).deleted
    }
}

# -----------------------------
# RUN EVERYTHING
# -----------------------------

$result = @{
    CREATE       = Measure-Request "POST" "$baseUrl/create" @{ title="test" } $serverPid
    READ         = Measure-Request "GET"  "$baseUrl"         $null           $serverPid
}

# Fetch all IDs
$html = Invoke-WebRequest "$baseUrl"
$allIds = ([regex]::Matches($html.Content, "\b\d+\b") | ForEach-Object { $_.Value }) | Select-Object -Unique

if ($allIds.Count -gt 0) {
    $lastId = $allIds[-1]
    $result.UPDATE = Measure-Request "POST" "$baseUrl/edit/$lastId" @{ title="upd" } $serverPid
    $result.DELETE = Measure-Request "POST" "$baseUrl/delete/$lastId" $null $serverPid
} else {
    $result.UPDATE = @{ error = "no id" }
    $result.DELETE = @{ error = "no id" }
}

# Bulk ops
$result.BULK_CREATE = Bulk-Create $baseUrl $bulkCount $serverPid

$html = Invoke-WebRequest "$baseUrl"
$allIds = ([regex]::Matches($html.Content, "\b\d+\b") | ForEach-Object { $_.Value }) | Select-Object -Unique
$bulkIds = $allIds[-$bulkCount..-1]

$result.BULK_READ   = Measure-Request "GET" "$baseUrl" $null $serverPid
$result.BULK_UPDATE = Bulk-Update $baseUrl $bulkIds $serverPid
$result.BULK_DELETE = Bulk-Delete $baseUrl $bulkIds $serverPid

# Output JSON
($result | ConvertTo-Json -Depth 6)
