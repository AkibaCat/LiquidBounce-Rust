param(
    [ValidateSet('hello', 'status', 'state', 'entities', 'inventory', 'screen', 'ui',
        'input-log', 'options-get', 'key', 'mouse', 'move', 'look', 'turn', 'click',
        'release', 'stop', 'wait', 'wait-ready')]
    [string] $Command = 'hello',
    [string[]] $Arguments = @(),
    [string] $Controller = 'D:\Develop\MCL\.references\horizon\nightly\horizon-windows-x64\horizon-ctl.exe',
    [string] $Socket = '',
    [ValidateRange(1, 60)]
    [int] $TimeoutSeconds = 5
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Controller -PathType Leaf)) {
    throw "Horizon controller not found: $Controller"
}

# The official controller handles the localhost TCP token/Unix socket.
# Commands operate on structured game state and widget IDs; no screen recognition.
$controlArguments = @('--timeout', $TimeoutSeconds.ToString())
if ($Socket) { $controlArguments += @('--socket', $Socket) }
$controlArguments += @($Command)
$controlArguments += $Arguments
& $Controller @controlArguments
exit $LASTEXITCODE
