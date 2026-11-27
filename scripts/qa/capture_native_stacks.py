#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Local Windows x64 diagnostic for explicitly selected QA processes.

Requires an explicitly selected liquidbounce-native-game.exe PID. Each thread
is resumed in a finally block immediately after copying its raw unwind frames.
Symbol lookup and JSON output happen only after that thread has been resumed.
Structs follow Windows SDK 10.0.26100.0 winnt.h, DbgHelp.h and TlHelp32.h.
No symbol server, network access, process launch, or process memory write.
"""

import argparse
import ctypes as C
from ctypes import wintypes as W
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import sys
import time

U64 = C.c_uint64
PTR = C.c_void_p


class Context(C.Structure):
    _fields_ = [(name, U64) for name in ('P1Home','P2Home','P3Home','P4Home','P5Home','P6Home')] + [
        ('ContextFlags', W.DWORD), ('MxCsr', W.DWORD),
    ] + [(name, W.WORD) for name in ('SegCs','SegDs','SegEs','SegFs','SegGs','SegSs')] + [
        ('EFlags', W.DWORD),
    ] + [(name, U64) for name in ('Dr0','Dr1','Dr2','Dr3','Dr6','Dr7',
        'Rax','Rcx','Rdx','Rbx','Rsp','Rbp','Rsi','Rdi','R8','R9','R10','R11','R12','R13','R14','R15','Rip')] + [
        ('FltSave', C.c_byte * 512), ('VectorRegister', C.c_byte * 416),
    ] + [(name, U64) for name in ('VectorControl','DebugControl','LastBranchToRip',
        'LastBranchFromRip','LastExceptionToRip','LastExceptionFromRip')]


class Address(C.Structure):
    _fields_ = [('Offset', U64), ('Segment', W.WORD), ('Mode', W.DWORD)]


class KdHelp(C.Structure):
    _fields_ = [('Thread', U64)] + [(n,W.DWORD) for n in
        ('ThCallbackStack','ThCallbackBStore','NextCallback','FramePointer')] + [(n,U64) for n in
        ('KiCallUserMode','KeUserCallbackDispatcher','SystemRangeStart','KiUserExceptionDispatcher','StackBase','StackLimit')] + [
        ('BuildVersion',W.DWORD), ('RetpolineStubFunctionTableSize',W.DWORD),
        ('RetpolineStubFunctionTable',U64), ('RetpolineStubOffset',W.DWORD),
        ('RetpolineStubSize',W.DWORD), ('Reserved0',U64*2)]


class StackFrame(C.Structure):
    _fields_ = [(n,Address) for n in ('AddrPC','AddrReturn','AddrFrame','AddrStack','AddrBStore')] + [
        ('FuncTableEntry',PTR), ('Params',U64*4), ('Far',W.BOOL), ('Virtual',W.BOOL),
        ('Reserved',U64*3), ('KdHelp',KdHelp)]


class ThreadEntry(C.Structure):
    _fields_ = [(n,W.DWORD) for n in ('dwSize','cntUsage','th32ThreadID','th32OwnerProcessID')] + [
        ('tpBasePri',W.LONG), ('tpDeltaPri',W.LONG), ('dwFlags',W.DWORD)]


class ModuleEntry(C.Structure):
    _fields_ = [(n,W.DWORD) for n in ('dwSize','th32ModuleID','th32ProcessID','GlblcntUsage','ProccntUsage')] + [
        ('modBaseAddr',PTR), ('modBaseSize',W.DWORD), ('hModule',W.HMODULE),
        ('szModule',W.WCHAR*256), ('szExePath',W.WCHAR*260)]


class SymbolInfo(C.Structure):
    _fields_ = [('SizeOfStruct',W.ULONG),('TypeIndex',W.ULONG),('Reserved',U64*2),
        ('Index',W.ULONG),('Size',W.ULONG),('ModBase',U64),('Flags',W.ULONG),('Value',U64),
        ('Address',U64)] + [(n,W.ULONG) for n in ('Register','Scope','Tag','NameLen','MaxNameLen')] + [('Name',C.c_char*1)]


class LineInfo(C.Structure):
    _fields_ = [('SizeOfStruct',W.DWORD),('Key',PTR),('LineNumber',W.DWORD),('FileName',C.c_char_p),('Address',U64)]


def binding(dll, name, result, args):
    func = getattr(dll,name)
    func.restype,func.argtypes = result,args
    return func


def fail(label):
    error=C.get_last_error()
    return OSError(error, f'{label}: {C.FormatError(error).strip()}')


def setup():
    if sys.platform != 'win32' or C.sizeof(PTR) != 8:
        raise RuntimeError('Use Windows x64 Python; the default installed Python on this host is x86.')
    sizes={cls.__name__:C.sizeof(cls) for cls in (Context,Address,KdHelp,StackFrame,ThreadEntry,ModuleEntry,SymbolInfo,LineInfo)}
    expected={'Context':1232,'Address':16,'KdHelp':112,'StackFrame':264,'ThreadEntry':28,'ModuleEntry':1080,'SymbolInfo':88,'LineInfo':40}
    if sizes != expected or Context.Rip.offset != 248 or Context.Rsp.offset != 152:
        raise RuntimeError(f'Unexpected Windows SDK structure layout: {sizes}')
    system=Path(os.environ.get('SystemRoot',r'C:\Windows'))/'System32'
    k=C.WinDLL(str(system/'kernel32.dll'),use_last_error=True)
    d=C.WinDLL(str(system/'dbghelp.dll'),use_last_error=True)
    for name,result,args in [
        ('OpenProcess',W.HANDLE,[W.DWORD,W.BOOL,W.DWORD]),
        ('CloseHandle',W.BOOL,[W.HANDLE]),
        ('QueryFullProcessImageNameW',W.BOOL,[W.HANDLE,W.DWORD,W.LPWSTR,C.POINTER(W.DWORD)]),
        ('IsWow64Process2',W.BOOL,[W.HANDLE,C.POINTER(W.WORD),C.POINTER(W.WORD)]),
        ('CreateToolhelp32Snapshot',W.HANDLE,[W.DWORD,W.DWORD]),
        ('Thread32First',W.BOOL,[W.HANDLE,C.POINTER(ThreadEntry)]),
        ('Thread32Next',W.BOOL,[W.HANDLE,C.POINTER(ThreadEntry)]),
        ('Module32FirstW',W.BOOL,[W.HANDLE,C.POINTER(ModuleEntry)]),
        ('Module32NextW',W.BOOL,[W.HANDLE,C.POINTER(ModuleEntry)]),
        ('OpenThread',W.HANDLE,[W.DWORD,W.BOOL,W.DWORD]),
        ('SuspendThread',W.DWORD,[W.HANDLE]), ('ResumeThread',W.DWORD,[W.HANDLE]),
        ('GetThreadContext',W.BOOL,[W.HANDLE,PTR]),
        ('GetThreadDescription',W.LONG,[W.HANDLE,C.POINTER(PTR)]),
        ('LocalFree',PTR,[PTR]),
        ('GetExitCodeProcess',W.BOOL,[W.HANDLE,C.POINTER(W.DWORD)]),
    ]:
        binding(k,name,result,args)
    for name,result,args in [
        ('SymSetOptions',W.DWORD,[W.DWORD]),
        ('SymInitializeW',W.BOOL,[W.HANDLE,W.LPCWSTR,W.BOOL]),
        ('SymCleanup',W.BOOL,[W.HANDLE]),
        ('SymFunctionTableAccess64',PTR,[W.HANDLE,U64]),
        ('SymGetModuleBase64',U64,[W.HANDLE,U64]),
        ('StackWalk64',W.BOOL,[W.DWORD,W.HANDLE,W.HANDLE,C.POINTER(StackFrame),PTR,PTR,PTR,PTR,PTR]),
        ('SymFromAddr',W.BOOL,[W.HANDLE,U64,C.POINTER(U64),C.POINTER(SymbolInfo)]),
        ('SymGetLineFromAddr64',W.BOOL,[W.HANDLE,U64,C.POINTER(W.DWORD),C.POINTER(LineInfo)]),
    ]:
        binding(d,name,result,args)
    return k,d,sizes


def snapshot_items(k,pid,modules=False):
    handle=k.CreateToolhelp32Snapshot(0x18 if modules else 0x04,pid if modules else 0)
    if handle in (None,C.c_void_p(-1).value):
        raise fail('CreateToolhelp32Snapshot')
    entry=ModuleEntry() if modules else ThreadEntry()
    entry.dwSize=C.sizeof(entry)
    first=k.Module32FirstW if modules else k.Thread32First
    nxt=k.Module32NextW if modules else k.Thread32Next
    values=[]
    try:
        ok=first(handle,C.byref(entry))
        if not ok:
            raise fail('Module32FirstW' if modules else 'Thread32First')
        while ok:
            if modules:
                values.append({'name':entry.szModule,'path':entry.szExePath,
                    'base':int(entry.modBaseAddr or 0),'size':int(entry.modBaseSize)})
            elif entry.th32OwnerProcessID == pid:
                values.append(int(entry.th32ThreadID))
            ok=nxt(handle,C.byref(entry))
        return values
    finally:
        k.CloseHandle(handle)


def raw_thread(k,d,process,tid,max_frames,seconds):
    result={'threadId':tid,'frames':[],'resumed':False}
    thread=k.OpenThread(0x0002|0x0008|0x0040,False,tid)
    if not thread:
        result['error']=str(fail('OpenThread'))
        return result
    suspended=False
    started=None
    try:
        desc=PTR()
        if k.GetThreadDescription(thread,C.byref(desc)) == 0 and desc:
            try:
                result['description']=C.wstring_at(desc)
            finally:
                k.LocalFree(desc)
        # CONTEXT requires 16-byte alignment; ctypes only guarantees 8 here.
        storage=C.create_string_buffer(C.sizeof(Context)+15)
        address=(C.addressof(storage)+15)&~15
        ctx=Context.from_address(address)
        ctx.ContextFlags=0x00100003  # AMD64 CONTROL | INTEGER
        previous=k.SuspendThread(thread)
        if previous == 0xffffffff:
            raise fail('SuspendThread')
        suspended=True
        started=time.perf_counter()
        result['previousSuspendCount']=int(previous)
        if not k.GetThreadContext(thread,address):
            raise fail('GetThreadContext')
        result['registers']={n:hex(getattr(ctx,n)) for n in ('Rip','Rsp','Rbp')}
        frame=StackFrame()
        for field,offset in [('AddrPC',ctx.Rip),('AddrStack',ctx.Rsp),('AddrFrame',ctx.Rbp)]:
            getattr(frame,field).Offset=offset
            getattr(frame,field).Mode=3  # AddrModeFlat
        initial=(int(frame.AddrPC.Offset),int(frame.AddrStack.Offset))
        result['frames'].append({'address':initial[0],'stackPointer':initial[1]})
        seen={initial}
        for step in range(max_frames):
            if time.perf_counter()-started > seconds:
                result['truncated']='per-thread suspension budget'
                break
            C.set_last_error(0)
            if not d.StackWalk64(0x8664,process,thread,C.byref(frame),address,None,
                C.cast(d.SymFunctionTableAccess64,PTR),C.cast(d.SymGetModuleBase64,PTR),None):
                result['stackWalkLastError']=C.get_last_error()
                break
            key=(int(frame.AddrPC.Offset),int(frame.AddrStack.Offset))
            if not key[0]:
                break
            if key in seen:
                # The first StackWalk64 can return the initial context frame.
                if step == 0:
                    continue
                break
            seen.add(key)
            result['frames'].append({'address':key[0],'stackPointer':key[1]})
            if len(result['frames']) >= max_frames:
                result['truncated']='frame limit'
                break
    except Exception as error:
        result['error']=str(error)
    finally:
        if suspended:
            # Balance precisely our single SuspendThread, preserving prior count.
            resume=k.ResumeThread(thread)
            result['resumed']=resume != 0xffffffff
            if resume == 0xffffffff:
                result['resumeError']=str(fail('ResumeThread'))
            result['suspendDurationMs']=round((time.perf_counter()-started)*1000,3)
        k.CloseHandle(thread)
    return result


def symbolize(d,process,frame,modules):
    address=frame['address']
    frame['addressHex']=hex(address)
    module=next((m for m in modules if m['base'] <= address < m['base']+m['size']),None)
    if module:
        frame['module']=module['name']
        frame['moduleOffset']=hex(address-module['base'])
    buffer=C.create_string_buffer(C.sizeof(SymbolInfo)+4096)
    symbol=SymbolInfo.from_buffer(buffer)
    symbol.SizeOfStruct=C.sizeof(SymbolInfo)
    symbol.MaxNameLen=4096
    displacement=U64()
    if d.SymFromAddr(process,address,C.byref(displacement),C.byref(symbol)):
        name=C.string_at(C.addressof(buffer)+SymbolInfo.Name.offset,symbol.NameLen)
        frame['function']=name.decode('utf-8',errors='replace')
        frame['functionOffset']=hex(displacement.value)
    else:
        frame['symbolError']=C.get_last_error()
    line=LineInfo()
    line.SizeOfStruct=C.sizeof(line)
    line_offset=W.DWORD()
    if d.SymGetLineFromAddr64(process,address,C.byref(line_offset),C.byref(line)):
        frame['source']={'file':line.FileName.decode('utf-8',errors='replace') if line.FileName else '',
            'line':int(line.LineNumber),'displacement':int(line_offset.value)}


def capture(args,k,d):
    if args.pid <= 4 or args.pid == os.getpid():
        raise RuntimeError('Refusing system/self PID.')
    process=k.OpenProcess(0x0400|0x0010,False,args.pid)
    if not process:
        raise fail('OpenProcess')
    initialized=False
    try:
        image=C.create_unicode_buffer(32768)
        length=W.DWORD(len(image))
        if not k.QueryFullProcessImageNameW(process,0,image,C.byref(length)):
            raise fail('QueryFullProcessImageNameW')
        if Path(image.value).name.lower() != 'liquidbounce-native-game.exe':
            raise RuntimeError(f'Refusing unexpected executable: {image.value}')
        if args.expected_image and os.path.normcase(os.path.abspath(image.value)) != os.path.normcase(str(args.expected_image.resolve())):
            raise RuntimeError(f'Target path differs from --expected-image: {image.value}')
        process_machine,native_machine=W.WORD(),W.WORD()
        if not k.IsWow64Process2(process,C.byref(process_machine),C.byref(native_machine)):
            raise fail('IsWow64Process2')
        if process_machine.value != 0 or native_machine.value != 0x8664:
            raise RuntimeError('Target must be native AMD64.')
        modules=snapshot_items(k,args.pid,modules=True)
        thread_ids=snapshot_items(k,args.pid)
        symbol_paths=[str(p.resolve()) for p in args.symbols]
        # Explicit local paths only; ignore environment symbol servers and preload
        # symbols before suspending any thread (no deferred symbol load option).
        options=0x2|0x10|0x200|0x1000|0x80000
        d.SymSetOptions(options)
        symbol_started=time.perf_counter()
        if not d.SymInitializeW(process,';'.join(symbol_paths),True):
            raise fail('SymInitializeW')
        initialized=True
        report={'createdUtc':datetime.now(timezone.utc).isoformat(),'pid':args.pid,'image':image.value,
            'symbolSearchPaths':symbol_paths,'symbolOptions':hex(options),
            'symbolInitSeconds':round(time.perf_counter()-symbol_started,3),'modules':modules,'threads':[],
            'maxFrames':args.max_frames,'perThreadBudgetSeconds':args.thread_seconds}
        # All DbgHelp calls are serialized in this one Python thread.
        for tid in thread_ids:
            raw=raw_thread(k,d,process,tid,args.max_frames,args.thread_seconds)
            # Target thread is already restored before expensive symbol queries.
            for frame in raw['frames']:
                symbolize(d,process,frame,modules)
            report['threads'].append(raw)
        exit_code=W.DWORD()
        report['targetStillAlive']=bool(k.GetExitCodeProcess(process,C.byref(exit_code)) and exit_code.value==259)
        report['exitCode']=int(exit_code.value)
        report['allSuspendedThreadsResumed']=all(t.get('resumed') for t in report['threads'] if 'previousSuspendCount' in t)
        return report
    finally:
        if initialized:
            d.SymCleanup(process)
        k.CloseHandle(process)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pid',type=int,help='Explicit game PID selected by the operator; never auto-discovers a process.')
    parser.add_argument('--expected-image',type=Path,help='Optional exact executable path guard.')
    parser.add_argument('--output',type=Path,help='New diagnostic JSON file; existing files are not overwritten.')
    parser.add_argument('--symbols',type=Path,action='append',default=[])
    parser.add_argument('--max-frames',type=int,default=96,choices=range(1,257),metavar='1..256')
    parser.add_argument('--thread-seconds',type=float,default=0.15)
    parser.add_argument('--validate-only',action='store_true',help='Validate DLL exports and SDK structure sizes without opening any process.')
    args=parser.parse_args()
    k,d,sizes=setup()
    if args.validate_only:
        print(json.dumps({'validated':True,'structureSizes':sizes,'contextRipOffset':Context.Rip.offset,'contextRspOffset':Context.Rsp.offset}))
        return
    if not args.pid or not args.output:
        parser.error('--pid and --output are required for capture.')
    if not 0.01 <= args.thread_seconds <= 0.5:
        parser.error('--thread-seconds must be between 0.01 and 0.5.')
    if args.output.exists():
        parser.error('Output file already exists; select a new file.')
    if not args.symbols:
        args.symbols=[Path(__file__).resolve().parents[2]/'build/staging/horizon/target/debug']
    report=capture(args,k,d)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    with args.output.open('x',encoding='utf-8') as stream:
        json.dump(report,stream,ensure_ascii=False,indent=2)
        stream.write('\n')
    print(json.dumps({'output':str(args.output.resolve()),'pid':report['pid'],'threadCount':len(report['threads']),
        'targetStillAlive':report['targetStillAlive'],'allSuspendedThreadsResumed':report['allSuspendedThreadsResumed']}))


if __name__=='__main__':
    main()
