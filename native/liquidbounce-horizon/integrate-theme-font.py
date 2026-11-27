#!/usr/bin/env python3
"""Transactional real-browser theme-font binding; no replacement theme or test font setting."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-chest-stealer.py'))['Plan']
def prepare():
 p=Plan()
 p.edit('lib.rs','mod target_renderer_runtime;','mod target_renderer_runtime;\n#[cfg(feature="webview")]\nmod theme_font_runtime;')
 p.edit('browser.rs','    reload_held: bool,','    reload_held: bool,\n    native_fonts:crate::theme_font_runtime::ThemeFontMonitor,')
 p.edit('browser.rs','    let (backend, inbox) = QueuedBackend::channel(256, Duration::from_secs(8));','    let native_fonts=crate::theme_font_runtime::ThemeFontMonitor::new(config.theme_root.clone());\n    let (backend, inbox) = QueuedBackend::channel(256, Duration::from_secs(8));')
 p.edit('browser.rs','        reload_held: false,','        reload_held: false,\n        native_fonts,')
 anchor='        let mut browser = browser.lock().unwrap_or_else(|e| e.into_inner());\n        let language_generation = horizon_client::lang::generation();'
 p.edit('browser.rs',anchor,'        let mut browser = browser.lock().unwrap_or_else(|e| e.into_inner());\n        browser.native_fonts.refresh();\n        let language_generation = horizon_client::lang::generation();')
 p.edit('browser.rs','            browser.reload_held = true;','            browser.reload_held = true;\n            browser.native_fonts.reload();')
 return p
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');args=a.parse_args();p=prepare()
 if not args.check:p.commit()
 print('ThemeFont '+('validated'if args.check else'integrated')+f': {len(p.changed)} shared files')
