from pathlib import Path
HERE=Path(__file__).resolve().parent
HOOKS='''    pub(super) fn native_client_book_bot_snapshot(&self)->Option<String>{self.book_bot_snapshot()}
    pub(super) fn native_client_book_bot_selected(&self)->i32{self.book_bot_selected()}
    pub(super) fn native_client_book_bot_write(&mut self,settings:&str,root:&str,count:i32)->bool{self.book_bot_write(settings,root,count)}
'''
def apply(edit,once):
    for name,path in [('book_bot',HERE/'book-bot-game.rs'),('book_bot_content',HERE/'book-bot-content.rs')]:
        content=path.read_text(encoding='utf8');edit('crates/horizon-client/src/game/'+name+'.rs',lambda _,v=content:v)
    edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod books;\n','mod books;\nmod book_bot;\nmod book_bot_content;\n'))
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,HOOKS+'\n'+anchor))
    return {'hook_names':[],'helper_names':['client_book_bot_snapshot','client_book_bot_selected','client_book_bot_write']}
