"""NameProtect labels/descriptions in Simplified and Traditional Chinese."""
from pathlib import Path
import argparse,json
from display_locale_schema import put_module
HERE=Path(__file__).resolve().parent
TEXTS={
    'description':('Hide player names locally in chat, name tags, player lists and other drawn text.','在聊天、名牌、玩家列表及其他显示文本中隐藏玩家真实名称。','在聊天、名牌、玩家清單與其他顯示文字中隱藏玩家真實名稱。'),
    'replacement.description':('Replacement for your own username.','显示在自己名称位置的替换文本。','顯示於自身名稱位置的替換文字。'),
    'colorMode.description':('Color used when a styled text sequence replaces your own name.','替换带样式文本中的自身名称时使用的颜色。','替換帶樣式文字中的自身名稱時使用的顏色。'),
    'colorMode.static.description':('Use a fixed name color.','使用固定的名称颜色。','使用固定的名稱顏色。'),
    'colorMode.static.color.description':('Fixed replacement color.','名称替换后的固定颜色。','名稱替換後的固定顏色。'),
    'colorMode.rainbow.description':('Cycle the replacement hue every ten seconds.','名称颜色每十秒循环一次彩虹色相。','名稱顏色每十秒循環一次彩虹色相。'),
    'obfuscateFriends.description':('Replace friend names with their aliases, or numbered Friend names.','将好友名称替换为备注名称；没有备注时使用带编号的 Friend 名称。','將好友名稱替換為備註名稱；沒有備註時使用帶編號的 Friend 名稱。'),
    'obfuscateFriends.enabled.description':('Enable replacement of friend names.','启用好友名称替换。','啟用好友名稱替換。'),
    'obfuscateOthers.description':('Give other online players stable generated names.','为其他在线玩家显示根据其原名称固定生成的化名。','為其他線上玩家顯示依原名稱固定產生的化名。'),
    'obfuscateOthers.enabled.description':('Enable replacement of other online players.','启用其他在线玩家名称替换。','啟用其他線上玩家名稱替換。'),
}
for group,cn,tw in [('obfuscateFriends','好友','好友'),('obfuscateOthers','其他玩家','其他玩家')]:
    for suffix in ['colorMode','colorMode.static','colorMode.static.color','colorMode.rainbow']:
        value=TEXTS[suffix+'.description']
        TEXTS[f'{group}.{suffix}.description']=(value[0],value[1].replace('自身名称',cn+'名称'),value[2].replace('自身名稱',tw+'名稱'))


def update(folder):
    for locale,index in [('zh_cn',1),('zh_tw',2)]:
        path=folder/f'display_{locale}.json';data=json.loads(path.read_text(encoding='utf8'))
        put_module(data,'liquidbounce.module.nameProtect','NameProtect',{'':('名称保护','名稱保護')[index-1],**{k:v[index]for k,v in TEXTS.items()}},{k:v[0]for k,v in TEXTS.items()})
        for name,values in {'Replacement':('替换文本','替換文字'),'ObfuscateFriends':('隐藏好友名称','隱藏好友名稱'),'ObfuscateOthers':('隐藏其他玩家名称','隱藏其他玩家名稱')}.items():data['labels'][name]=values[index-1]
        path.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--folder',type=Path,default=HERE.parent/'liquidbounce-interop/resources/localization');update(parser.parse_args().folder)
