#!/usr/bin/env python3
"""Keep a real Windows process handle open and record an isolated game's exit."""
import argparse
import ctypes
from ctypes import wintypes
from datetime import datetime, timezone
import json
from pathlib import Path
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--run', type=Path, required=True)
p.add_argument('--seconds', type=int, default=1800)
args = p.parse_args()
launch = json.loads((args.run / 'launch-result.json').read_text(encoding='utf-8-sig'))
kernel = ctypes.WinDLL('kernel32', use_last_error=True)
kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
kernel.OpenProcess.restype = wintypes.HANDLE
kernel.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
kernel.GetExitCodeProcess.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
kernel.CloseHandle.argtypes = [wintypes.HANDLE]
handle = kernel.OpenProcess(0x100000 | 0x1000, False, launch['processId'])
if not handle:
    raise ctypes.WinError(ctypes.get_last_error())
result = {'processId':launch['processId'], 'openedUtc':datetime.now(timezone.utc).isoformat(),
          'method':'OpenProcess/WaitForSingleObject/GetExitCodeProcess', 'passed':False}
try:
    length = wintypes.DWORD(32768)
    filename = ctypes.create_unicode_buffer(length.value)
    if not kernel.QueryFullProcessImageNameW(handle, 0, filename, ctypes.byref(length)):
        raise ctypes.WinError(ctypes.get_last_error())
    if Path(filename.value).resolve() != Path(launch['game']).resolve():
        raise RuntimeError('Process identity does not match the isolated launch')
    result['executable'] = filename.value
    (args.run / 'process-monitor-open.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    deadline = time.monotonic() + args.seconds
    while time.monotonic() < deadline:
        status = kernel.WaitForSingleObject(handle, 1000)
        if status == 0:
            code = wintypes.DWORD()
            if not kernel.GetExitCodeProcess(handle, ctypes.byref(code)):
                raise ctypes.WinError(ctypes.get_last_error())
            result['exitCode'] = code.value
            result['passed'] = code.value == 0
            break
        if status != 258:
            raise ctypes.WinError(ctypes.get_last_error())
    else:
        result['error'] = 'Timed out waiting for official game exit'
finally:
    kernel.CloseHandle(handle)
    result['finishedUtc'] = datetime.now(timezone.utc).isoformat()
    (args.run / 'process-exit-validation.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
print(json.dumps(result))
raise SystemExit(0 if result['passed'] else 1)
