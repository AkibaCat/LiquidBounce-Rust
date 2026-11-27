#!/usr/bin/env python3
"""Merge GUICloser's original descriptions and complete setting help only."""
from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[2]
BASE='liquidbounce.module.guiCloser.'
HELP={
    'en_us':{
        'mode.description':'Matches tests the entire container title; Contains searches for any matching part. Both modes use Java regular expressions.',
        'filter.description':'Ordered Java regular expressions for container titles. The first matching expression prevents that screen from opening. The default is ^Vote$.',
        'printScreenTitle.description':'Print unmatched container titles in chat. Click the highlighted title to copy it. This setting is saved locally and shown in the interface, but omitted from public configuration exports.',
    },
    'zh_cn':{
        'mode.description':'Matches 匹配完整容器标题；Contains 搜索标题中的匹配部分。两种模式都使用 Java 正则表达式。',
        'filter.description':'按顺序检查容器标题的 Java 正则表达式。首个匹配项会阻止该界面打开，默认表达式为 ^Vote$。',
        'printScreenTitle.description':'在聊天中显示未匹配的容器标题，点击高亮标题即可复制。此设置会保存在本地并显示在界面中，但不会包含在公开配置导出中。',
    },
    'zh_tw':{
        'mode.description':'Matches 比對完整容器標題；Contains 搜尋標題中的相符部分。兩種模式都使用 Java 正規表示式。',
        'filter.description':'依序檢查容器標題的 Java 正規表示式。第一個相符項目會阻止該介面開啟，預設表示式為 ^Vote$。',
        'printScreenTitle.description':'在聊天中顯示未相符的容器標題，點擊醒目標題即可複製。此設定會儲存在本機並顯示於介面中，但不會包含在公開設定匯出中。',
    },
}
def main():
    for language in ['zh_cn','zh_tw']:
        target=ROOT/f'native/liquidbounce-interop/resources/localization/display_{language}.json'
        data=json.loads(target.read_text(encoding='utf8'))
        source=json.loads((ROOT/f'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/{language}.json').read_text(encoding='utf8'))
        data['descriptions'][BASE+'description']=source[BASE+'description']
        data['descriptions'].update({BASE+k:v for k,v in HELP[language].items()})
        data['labels'].update({'Mode':'模式','Filter':'过滤规则' if language=='zh_cn' else '篩選規則','PrintScreenTitle':'显示界面标题' if language=='zh_cn' else '顯示介面標題'})
        if language=='zh_cn':
            source=json.loads((ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/lang/en_us.json').read_text(encoding='utf8'))
            data['englishDescriptions'][BASE+'description']=source[BASE+'description']
            data['englishDescriptions'].update({BASE+k:v for k,v in HELP['en_us'].items()})
        target.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
if __name__=='__main__':main()
