from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    # Only the native title constants are needed; keep the full production
    # behavior in its engine crate rather than copying inventory dependencies.
    source=(HERE.parent/'liquidbounce-behaviors/src/chest_cleaner.rs').read_text(encoding='utf8')
    start=source.index('pub const TITLE_KEYS:')
    end=source.index('#[derive',start)
    edit('crates/horizon-client/src/game/chest_cleaner_titles.rs',lambda _:source[start:end])
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod books;\n','mod books;\nmod chest_cleaner_titles;\n'))
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'chest-cleaner-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    return {'hook_names':[],'helper_names':['client_chest_cleaner_context']}
